// Importações padrão para iteração e análise de caracteres
use std::iter::Peekable;
use std::str::Chars;

/// Representa uma expressão da linguagem Cmd
#[derive(Debug)]
pub enum Expr {
    Const(i32), // Um número inteiro literal, ex: 42
    Var(String), // Uma variável, ex: x
    OpBin {
        operador: String,     // Operador: +, -, *, /, <, >, ==
        esq: Box<Expr>,       // Operando da esquerda
        dir: Box<Expr>,       // Operando da direita
    },
}

/// Representa um comando da linguagem Cmd: if, while ou atribuição
#[derive(Debug)]
pub enum Cmd {
    If {
        cond: Expr,           // Condição do if
        then_cmds: Vec<Cmd>,  // Comandos do bloco verdadeiro
        else_cmds: Vec<Cmd>,  // Comandos do bloco falso
    },
    While {
        cond: Expr,           // Condição do loop
        body: Vec<Cmd>,       // Corpo do loop
    },
    Atrib {
        nome: String,         // Nome da variável
        expr: Expr,           // Expressão a ser atribuída
    },
}

/// Representa um programa completo na linguagem Cmd
#[derive(Debug)]
pub struct Programa {
    pub declaracoes: Vec<(String, Expr)>, // Declarações de variáveis no topo
    pub comandos: Vec<Cmd>,               // Comandos dentro do bloco
    pub retorno: Expr,                    // Expressão do return
}

/// Estrutura do parser com um iterador de caracteres com espiada (peek)
pub struct Parser<'a> {
    tokens: Peekable<Chars<'a>>,
}

impl<'a> Parser<'a> {
    /// Construtor do parser
    pub fn new(input: &'a str) -> Self {
        Parser {
            tokens: input.chars().peekable(),
        }
    }

    /// Lê o próximo caractere ignorando espaços
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

    /// Espia o próximo caractere ignorando espaços
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

    /// Parse completo de um programa Cmd
    pub fn parse_programa(&mut self) -> Result<Programa, String> {
        let mut decls = Vec::new();

        // Analisa as declarações (var = expr;)
        while let Some(c) = self.peek() {
            if !c.is_ascii_alphabetic() {
                break;
            }

            // Evita confundir palavras-chave (if, while) com identificadores
            let mut clone = self.tokens.clone();
            while let Some(&ch) = clone.peek() {
                if ch.is_whitespace() { clone.next(); } else { break }
            }
            if clone.next() == Some('i')
               && clone.next() == Some('f')
               && !matches!(clone.peek(), Some(x) if x.is_ascii_alphanumeric())
            {
                break;
            }
            let mut clone2 = self.tokens.clone();
            while let Some(&ch) = clone2.peek() {
                if ch.is_whitespace() { clone2.next(); } else { break }
            }
            if clone2.next() == Some('w')
               && clone2.next() == Some('h')
               && clone2.next() == Some('i')
               && clone2.next() == Some('l')
               && clone2.next() == Some('e')
               && !matches!(clone2.peek(), Some(x) if x.is_ascii_alphanumeric())
            {
                break;
            }

            // Declaração válida
            let var = self.parse_var()?;
            self.expect('=')?;
            let expr = self.parse_expr()?;
            self.expect(';')?;
            decls.push((var, expr));
        }

        // Bloco principal de comandos
        self.expect('{')?;
        let mut cmds = Vec::new();
        while self.peek() != Some('r') {
            cmds.push(self.parse_cmd()?);
        }

        // return <expr>;
        if !self.parse_kw("return")? {
            return Err("Esperado 'return'".into());
        }
        let ret_expr = self.parse_expr()?;
        self.expect(';')?;
        self.expect('}')?;

        Ok(Programa {
            declaracoes: decls,
            comandos: cmds,
            retorno: ret_expr,
        })
    }

    /// Analisa um comando (if, while, ou atribuição)
    fn parse_cmd(&mut self) -> Result<Cmd, String> {
        if self.match_kw("if")? {
            let cond = self.parse_expr()?;
            self.expect('{')?;
            let mut then_cmds = Vec::new();
            while self.peek() != Some('}') {
                then_cmds.push(self.parse_cmd()?);
            }
            self.expect('}')?;

            let mut else_cmds = Vec::new();
            if self.match_kw("else")? {
                self.expect('{')?;
                while self.peek() != Some('}') {
                    else_cmds.push(self.parse_cmd()?);
                }
                self.expect('}')?;
            }

            Ok(Cmd::If { cond, then_cmds, else_cmds })
        }
        else if self.match_kw("while")? {
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

    /// Analisa expressões com comparações (==, <, >)
    fn parse_expr(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_exp_a()?;

        while let Some(c) = self.peek() {
            let op = match c {
                '=' => {
                    self.next();
                    if self.next() == Some('=') { "==".into() }
                    else { return Err("Operador = mal formado".into()) }
                }
                '<' | '>' => self.next().unwrap().to_string(),
                _ => break,
            };
            let right = self.parse_exp_a()?;
            expr = Expr::OpBin {
                operador: op,
                esq: Box::new(expr),
                dir: Box::new(right),
            };
        }

        Ok(expr)
    }

    /// Analisa expressões com + e -
    fn parse_exp_a(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_exp_m()?;
        while let Some(c) = self.peek() {
            if c == '+' || c == '-' {
                let op = self.next().unwrap().to_string();
                let right = self.parse_exp_m()?;
                expr = Expr::OpBin {
                    operador: op,
                    esq: Box::new(expr),
                    dir: Box::new(right),
                };
            } else { break; }
        }
        Ok(expr)
    }

    /// Analisa expressões com * e /
    fn parse_exp_m(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_prim()?;
        while let Some(c) = self.peek() {
            if c == '*' || c == '/' {
                let op = self.next().unwrap().to_string();
                let right = self.parse_prim()?;
                expr = Expr::OpBin {
                    operador: op,
                    esq: Box::new(expr),
                    dir: Box::new(right),
                };
            } else { break; }
        }
        Ok(expr)
    }

    /// Analisa primitivos: número, variável ou expressão entre parênteses
    fn parse_prim(&mut self) -> Result<Expr, String> {
        match self.peek() {
            Some(c) if c.is_ascii_digit()     => self.parse_const(),
            Some(c) if c.is_ascii_alphabetic() => {
                let v = self.parse_var()?;
                Ok(Expr::Var(v))
            }
            Some('(') => {
                self.next();
                let e = self.parse_expr()?;
                self.expect(')')?;
                Ok(e)
            }
            Some(c) => Err(format!("Token inesperado '{}'", c)),
            None    => Err("Fim inesperado de entrada".into()),
        }
    }

    /// Analisa um número inteiro
    fn parse_const(&mut self) -> Result<Expr, String> {
        let mut num = 0;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                num = num * 10 + c.to_digit(10).unwrap() as i32;
                self.next();
            } else {
                break;
            }
        }
        Ok(Expr::Const(num))
    }

    /// Analisa um identificador (nome de variável)
    fn parse_var(&mut self) -> Result<String, String> {
        let mut name = String::new();
        if let Some(c) = self.peek() {
            if c.is_ascii_alphabetic() {
                name.push(self.next().unwrap());
            } else {
                return Err("Identificador inválido".into());
            }
        } else {
            return Err("Fim inesperado ao tentar ler identificador".into());
        }
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() {
                name.push(self.next().unwrap());
            } else {
                break;
            }
        }
        Ok(name)
    }

    /// Verifica se o próximo token é a palavra-chave exata
    fn parse_kw(&mut self, kw: &str) -> Result<bool, String> {
        let mut clone = self.tokens.clone();
        while let Some(&c) = clone.peek() {
            if c.is_whitespace() { clone.next(); } else { break }
        }
        for expected in kw.chars() {
            if Some(expected) != clone.next() {
                return Ok(false);
            }
        }
        if let Some(&next) = clone.peek() {
            if next.is_ascii_alphanumeric() {
                return Ok(false);
            }
        }
        self.tokens = clone;
        Ok(true)
    }

    /// Consome a palavra-chave, se ela estiver presente
    fn match_kw(&mut self, kw: &str) -> Result<bool, String> {
        Ok(self.parse_kw(kw)?)
    }

    /// Espera um caractere específico (ex: ';', '{')
    fn expect(&mut self, ch: char) -> Result<(), String> {
        match self.next() {
            Some(c) if c == ch => Ok(()),
            Some(c)           => Err(format!("Esperado '{}', encontrado '{}'", ch, c)),
            None              => Err(format!("Esperado '{}', mas fim de entrada", ch)),
        }
    }
}

/// testes
#[cfg(test)]
mod tests {
    use super::*;

    fn parse_ok(source: &str) {
        let mut parser = Parser::new(source);
        assert!(parser.parse_programa().is_ok(), "Esperado sucesso no parsing para:\n{}", source);
    }

    fn parse_err(source: &str) {
        let mut parser = Parser::new(source);
        assert!(parser.parse_programa().is_err(), "Esperado erro no parsing para:\n{}", source);
    }

    #[test]
    fn test_declaracoes_e_return() {
        parse_ok("x = 1; y = 2; { return x; }");
    }

    #[test]
    fn test_if_verdadeiro() {
        parse_ok("a = 4; { if a < 10 { a = 1; } else { a = 0; } return a; }");
    }

    #[test]
    fn test_if_falso() {
        parse_ok("a = 20; { if a < 10 { a = 1; } else { a = 0; } return a; }");
    }

    #[test]
    fn test_while_loop() {
        parse_ok("a = 0; { while a < 5 { a = a + 1; } return a; }");
    }

    #[test]
    fn test_expressao_composta() {
        parse_ok("a = 1; b = 2; c = 3; { d = (a + b) * c; return d; }");
    }

    #[test]
    fn test_comparacao_operadores() {
        parse_ok("{ a = 1 == 1; b = 2 < 3; c = 4 > 1; return c; }");
    }

    #[test]
    fn test_falta_chave() {
        parse_err("a = 1; b = 2; return a + b;");
    }

    #[test]
    fn test_return_faltando() {
        parse_err("x = 1; { x = x + 1; }");
    }

    #[test]
    fn test_identificador_invalido() {
        parse_err("1x = 1; { return 0; }");
    }
}
