// Importa funcionalidades para iterar com "olhar à frente" sobre os caracteres
use std::iter::Peekable;
// Importa o tipo que representa uma sequência de caracteres
use std::str::Chars;

// Enumeração que representa uma expressão
#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Const(i32), // Constante inteira
    Var(String), // Variável com nome
    OpBin {
        operador: String,       // Operador binário como "+", "*", etc.
        esq: Box<Expr>,         // Expressão à esquerda
        dir: Box<Expr>,         // Expressão à direita
    },
    Call {
        nome: String,           // Nome da função
        args: Vec<Expr>,        // Argumentos da função
    },
}

// Enumeração que representa comandos da linguagem
#[derive(Debug, PartialEq, Clone)]
pub enum Cmd {
    If { cond: Expr, then_cmds: Vec<Cmd>, else_cmds: Vec<Cmd> }, // Comando condicional
    While { cond: Expr, body: Vec<Cmd> },                         // Comando de repetição
    Atrib { nome: String, expr: Expr },                           // Atribuição de valor
}

// Representa a definição de uma função
#[derive(Debug, PartialEq, Clone)]
pub struct FunDecl {
    pub nome: String,                // Nome da função
    pub parametros: Vec<String>,    // Parâmetros da função
    pub variaveis: Vec<(String, Expr)>, // Variáveis locais e seus valores iniciais
    pub comandos: Vec<Cmd>,         // Corpo da função
    pub retorno: Expr,              // Expressão de retorno
}

// Representa o programa completo
#[derive(Debug, PartialEq, Clone)]
pub struct Programa {
    pub globais: Vec<(String, Expr)>, // Variáveis globais
    pub funcoes: Vec<FunDecl>,        // Lista de funções definidas
    pub principal: Vec<Cmd>,          // Comandos principais (main)
    pub retorno: Expr,                // Valor de retorno do main
}

// Estrutura do parser: recebe os caracteres da entrada
pub struct Parser<'a> {
    tokens: Peekable<Chars<'a>>, // Iterador com capacidade de espiar o próximo caractere
}

impl<'a> Parser<'a> {
    // Cria uma nova instância do parser
    pub fn new(input: &'a str) -> Self {
        Parser {
            tokens: input.chars().peekable(),
        }
    }

    // Consome e retorna o próximo caractere não branco
    fn next(&mut self) -> Option<char> {
        while let Some(&c) = self.tokens.peek() {
            if c.is_whitespace() {
                self.tokens.next();
            } else {
                break;
            }
        }
        self.tokens.next()
    }

    // Espia o próximo caractere não branco sem consumi-lo
    fn peek(&mut self) -> Option<char> {
        let mut clone = self.tokens.clone();
        while let Some(&c) = clone.peek() {
            if c.is_whitespace() {
                clone.next();
            } else {
                break;
            }
        }
        clone.peek().copied()
    }

    // Inicia o parsing de um programa completo
    pub fn parse_programa(&mut self) -> Result<Programa, String> {
        let mut globais = Vec::new();
        let mut funcoes = Vec::new();

        // Processa as variáveis globais e funções até encontrar o main
        loop {
            if self.parse_kw("main")? {
                break;
            } else if self.parse_kw("var")? {
                let nome = self.parse_var()?;
                self.expect('=')?;
                let expr = self.parse_expr()?;
                self.expect(';')?;
                globais.push((nome, expr));
            } else if self.parse_kw("fun")? {
                funcoes.push(self.parse_fundecl()?);
            } else {
                return Err("Esperado 'fun', 'var' ou 'main'".into());
            }
        }

        // Processa o bloco principal do programa
        self.expect('{')?;
        let mut cmds = Vec::new();
        while !self.parse_kw("return")? {
            cmds.push(self.parse_cmd()?);
        }
        let retorno = self.parse_expr()?;
        self.expect(';')?;
        self.expect('}')?;

        Ok(Programa {
            globais,
            funcoes,
            principal: cmds,
            retorno,
        })
    }

    // Faz o parsing de uma função
    fn parse_fundecl(&mut self) -> Result<FunDecl, String> {
        let nome = self.parse_var()?;
        self.expect('(')?;
        let mut parametros = Vec::new();
        if self.peek() != Some(')') {
            parametros.push(self.parse_var()?);
            while self.peek() == Some(',') {
                self.next();
                parametros.push(self.parse_var()?);
            }
        }
        self.expect(')')?;
        self.expect('{')?;

        let mut variaveis = Vec::new();
        while self.parse_kw("var")? {
            let nome = self.parse_var()?;
            self.expect('=')?;
            let expr = self.parse_expr()?;
            self.expect(';')?;
            variaveis.push((nome, expr));
        }

        let mut comandos = Vec::new();
        while !self.parse_kw("return")? {
            comandos.push(self.parse_cmd()?);
        }
        let retorno = self.parse_expr()?;
        self.expect(';')?;
        self.expect('}')?;

        Ok(FunDecl {
            nome,
            parametros,
            variaveis,
            comandos,
            retorno,
        })
    }

    // Faz o parsing de um comando (if, while ou atribuição)
    fn parse_cmd(&mut self) -> Result<Cmd, String> {
        if self.parse_kw("if")? {
            let cond = self.parse_expr()?;
            self.expect('{')?;
            let mut then_cmds = Vec::new();
            while self.peek() != Some('}') {
                then_cmds.push(self.parse_cmd()?);
            }
            self.expect('}')?;
            self.expect_kw("else")?;
            self.expect('{')?;
            let mut else_cmds = Vec::new();
            while self.peek() != Some('}') {
                else_cmds.push(self.parse_cmd()?);
            }
            self.expect('}')?;
            Ok(Cmd::If { cond, then_cmds, else_cmds })
        } else if self.parse_kw("while")? {
            let cond = self.parse_expr()?;
            self.expect('{')?;
            let mut body = Vec::new();
            while self.peek() != Some('}') {
                body.push(self.parse_cmd()?);
            }
            self.expect('}')?;
            Ok(Cmd::While { cond, body })
        } else {
            let nome = self.parse_var()?;
            self.expect('=')?;
            let expr = self.parse_expr()?;
            self.expect(';')?;
            Ok(Cmd::Atrib { nome, expr })
        }
    }

    // Parsing de uma expressão, incluindo operadores relacionais
    fn parse_expr(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_exp_a()?;

        while let Some(op) = self.peek() {
            let operador = match op {
                '=' => {
                    self.next();
                    if self.next() == Some('=') { "==".into() } else {
                        return Err("Erro: operador = mal formado".into());
                    }
                }
                '<' | '>' => self.next().unwrap().to_string(),
                _ => break,
            };
            let dir = self.parse_exp_a()?;
            expr = Expr::OpBin {
                operador,
                esq: Box::new(expr),
                dir: Box::new(dir),
            };
        }
        Ok(expr)
    }

    // Parsing de expressões aditivas (soma/subtração)
    fn parse_exp_a(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_exp_m()?;
        while let Some(op) = self.peek() {
            if op == '+' || op == '-' {
                let op = self.next().unwrap().to_string();
                let dir = self.parse_exp_m()?;
                expr = Expr::OpBin {
                    operador: op,
                    esq: Box::new(expr),
                    dir: Box::new(dir),
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    // Parsing de expressões multiplicativas (multiplicação/divisão)
    fn parse_exp_m(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_prim()?;
        while let Some(op) = self.peek() {
            if op == '*' || op == '/' {
                let op = self.next().unwrap().to_string();
                let dir = self.parse_prim()?;
                expr = Expr::OpBin {
                    operador: op,
                    esq: Box::new(expr),
                    dir: Box::new(dir),
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    // Parsing de expressões primárias (número, variável, chamada, parênteses)
    fn parse_prim(&mut self) -> Result<Expr, String> {
        match self.peek() {
            Some(c) if c.is_ascii_digit() => self.parse_const(),
            Some(c) if c.is_ascii_alphabetic() => {
                let nome = self.parse_var()?;
                if self.peek() == Some('(') {
                    self.next();
                    let mut args = Vec::new();
                    if self.peek() != Some(')') {
                        args.push(self.parse_expr()?);
                        while self.peek() == Some(',') {
                            self.next();
                            args.push(self.parse_expr()?);
                        }
                    }
                    self.expect(')')?;
                    Ok(Expr::Call { nome, args })
                } else {
                    Ok(Expr::Var(nome))
                }
            }
            Some('(') => {
                self.next();
                let e = self.parse_expr()?;
                self.expect(')')?;
                Ok(e)
            }
            Some(c) => Err(format!("Token inesperado: '{}'", c)),
            None => Err("Fim inesperado".into()),
        }
    }

    // Parsing de constantes inteiras
    fn parse_const(&mut self) -> Result<Expr, String> {
        let mut valor = 0;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                valor = valor * 10 + c.to_digit(10).unwrap() as i32;
                self.next();
            } else {
                break;
            }
        }
        Ok(Expr::Const(valor))
    }

    // Parsing de identificadores (nomes de variáveis ou funções)
    fn parse_var(&mut self) -> Result<String, String> {
        let mut nome = String::new();
        if let Some(c) = self.peek() {
            if c.is_ascii_alphabetic() {
                nome.push(self.next().unwrap());
            } else {
                return Err("Esperado identificador".into());
            }
        }
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() {
                nome.push(self.next().unwrap());
            } else {
                break;
            }
        }
        Ok(nome)
    }

    // Tenta fazer o parsing de uma palavra-chave
    fn parse_kw(&mut self, kw: &str) -> Result<bool, String> {
        let mut clone = self.tokens.clone();
        while let Some(&c) = clone.peek() {
            if c.is_whitespace() { clone.next(); } else { break; }
        }
        for ch in kw.chars() {
            if Some(ch) != clone.next() {
                return Ok(false);
            }
        }
        if let Some(&next) = clone.peek() {
            if next.is_ascii_alphanumeric() {
                return Ok(false); // Evita confundir com prefixos de identificadores
            }
        }
        self.tokens = clone;
        Ok(true)
    }

    // Espera obrigatoriamente que uma palavra-chave apareça
    fn expect_kw(&mut self, kw: &str) -> Result<(), String> {
        if self.parse_kw(kw)? {
            Ok(())
        } else {
            Err(format!("Esperado '{}'", kw))
        }
    }

    // Espera obrigatoriamente por um caractere específico
    fn expect(&mut self, ch: char) -> Result<(), String> {
        match self.next() {
            Some(c) if c == ch => Ok(()),
            Some(c) => Err(format!("Esperado '{}', mas encontrou '{}'", ch, c)),
            None => Err(format!("Esperado '{}', mas fim da entrada", ch)),
        }
    }
}

//////////////
/// TESTES ///
//////////////
#[cfg(test)]
mod tests {
    use super::*;

    // Testa o parsing de uma expressão constante (valor numérico)
    #[test]
    fn test_parse_const() {
        let mut parser = Parser::new("123");
        let expr = parser.parse_expr().unwrap();
        // Verifica se a expressão parseada é uma constante com o valor 123
        assert_eq!(expr, Expr::Const(123));
    }

    // Testa o parsing de uma variável simples
    #[test]
    fn test_parse_var() {
        let mut parser = Parser::new("x");
        let expr = parser.parse_expr().unwrap();
        // Verifica se a expressão parseada é uma variável "x"
        assert_eq!(expr, Expr::Var("x".into()));
    }

    // Testa o parsing de uma operação binária de adição
    #[test]
    fn test_parse_opbin() {
        let mut parser = Parser::new("2 + 3");
        let expr = parser.parse_expr().unwrap();
        // Verifica se a expressão parseada é uma operação binária de adição entre 2 e 3
        assert_eq!(
            expr,
            Expr::OpBin {
                operador: "+".into(),
                esq: Box::new(Expr::Const(2)),
                dir: Box::new(Expr::Const(3)),
            }
        );
    }

    // Testa o parsing de uma chamada de função
    #[test]
    fn test_parse_call() {
        let mut parser = Parser::new("soma(1, x)");
        let expr = parser.parse_expr().unwrap();
        // Verifica se a expressão parseada é uma chamada de função "soma" com argumentos 1 e "x"
        assert_eq!(
            expr,
            Expr::Call {
                nome: "soma".into(),
                args: vec![Expr::Const(1), Expr::Var("x".into())],
            }
        );
    }

    // Testa o parsing de um comando de atribuição
    #[test]
    fn test_parse_cmd_atrib() {
        let mut parser = Parser::new("x = 42;");
        let cmd = parser.parse_cmd().unwrap();
        // Verifica se o comando parseado é uma atribuição para a variável "x" com valor 42
        assert_eq!(
            cmd,
            Cmd::Atrib {
                nome: "x".into(),
                expr: Expr::Const(42),
            }
        );
    }

    // Testa o parsing de um comando if-else
    #[test]
    fn test_parse_if_else() {
        let mut parser = Parser::new("if 1 { x = 2; } else { x = 3; }");
        let cmd = parser.parse_cmd().unwrap();
        match cmd {
            Cmd::If { cond, then_cmds, else_cmds } => {
                // Verifica se a condição do if é 1
                assert_eq!(cond, Expr::Const(1));
                // Verifica se os comandos dentro do bloco "then" e "else" envolvem a variável "x"
                assert!(matches!(&then_cmds[0], Cmd::Atrib { nome, .. } if nome == "x"));
                assert!(matches!(&else_cmds[0], Cmd::Atrib { nome, .. } if nome == "x"));
            }
            _ => panic!("Esperado Cmd::If"),
        }
    }

    // Testa o parsing de um comando while
    #[test]
    fn test_parse_while() {
        let mut parser = Parser::new("while x { y = y + 1; }");
        let cmd = parser.parse_cmd().unwrap();
        match cmd {
            Cmd::While { cond, body } => {
                // Verifica se a condição do while é a variável "x"
                assert_eq!(cond, Expr::Var("x".into()));
                // Verifica se o corpo do while envolve a variável "y"
                assert!(matches!(&body[0], Cmd::Atrib { nome, .. } if nome == "y"));
            }
            _ => panic!("Esperado Cmd::While"),
        }
    }
}
