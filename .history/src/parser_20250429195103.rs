use std::iter::{Peekable, Filter};
use std::str::Chars;

#[derive(Debug)]
pub enum Expr {
    Const(i32),
    Var(String),
    OpBin {
        operador: String,
        esq: Box<Expr>,
        dir: Box<Expr>,
    },
}

#[derive(Debug)]
pub enum Cmd {
    If {
        cond: Expr,
        then_cmds: Vec<Cmd>,
        else_cmds: Vec<Cmd>,
    },
    While {
        cond: Expr,
        body: Vec<Cmd>,
    },
    Atrib {
        nome: String,
        expr: Expr,
    },
}

#[derive(Debug)]
pub struct Programa {
    pub declaracoes: Vec<(String, Expr)>,
    pub comandos: Vec<Cmd>,
    pub retorno: Expr,
}

pub struct Parser<'a> {
    tokens: Peekable<Filter<Chars<'a>, fn(&char) -> bool>>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        fn not_whitespace(c: &char) -> bool {
            !c.is_whitespace()
        }

        Parser {
            tokens: input.chars().filter(not_whitespace as fn(&char) -> bool).peekable(),
        }
    }

    fn next(&mut self) -> Option<char> {
        self.tokens.next()
    }

    fn peek(&mut self) -> Option<char> {
        self.tokens.peek().copied()
    }

    pub fn parse_programa(&mut self) -> Result<Programa, String> {
        let mut decls = Vec::new();

        while let Some(c) = self.peek() {
            if c.is_ascii_alphabetic() {
                let var = self.parse_var()?;
                self.expect('=')?;
                let expr = self.parse_expr()?;
                self.expect(';')?;
                decls.push((var, expr));
            } else {
                break;
            }
        }

        self.expect('{')?;

        let mut cmds = Vec::new();
        while self.peek() != Some('r') {
            cmds.push(self.parse_cmd()?);
        }

        let return_kw = self.parse_kw("return")?;
        if !return_kw {
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

    fn parse_cmd(&mut self) -> Result<Cmd, String> {
        if self.match_kw("if")? {
            let cond = self.parse_expr()?;
            // Expect opening brace for the 'then' block
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
            // Expect opening brace for the 'while' body
            self.expect('{')?;
            let mut body = Vec::new();
            while self.peek() != Some('}') {
                body.push(self.parse_cmd()?);
            }
            self.expect('}')?;
            Ok(Cmd::While { cond, body })
        } else {
            let var = self.parse_var()?;
            self.expect('=')?;
            let expr = self.parse_expr()?;
            self.expect(';')?;
            Ok(Cmd::Atrib { nome: var, expr })
        }
    }
    

    fn parse_expr(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_exp_a()?;  // Aqui o foco é nas expressões aritméticas (+, -, *, /)
    
        // Agora lidamos com os operadores de comparação (menor, maior, igual)
        while let Some(c) = self.peek() {
            let op = match c {
                '=' => {
                    self.next();
                    if self.next() == Some('=') {
                        "==".to_string()  // Comparação de igualdade
                    } else {
                        return Err("Operador = mal formado".into());
                    }
                }
                '<' | '>' => {
                    self.next().unwrap().to_string()  // Processa os operadores de comparação (<, >)
                }
                _ => break,  // Se não for um operador de comparação, sai do loop
            };
    
            let right = self.parse_exp_a()?;  // A parte direita da comparação também é uma expressão
            expr = Expr::OpBin {
                operador: op,
                esq: Box::new(expr),
                dir: Box::new(right),
            };
        }
    
        Ok(expr)
    }
    
    
    

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
            } else {
                break;
            }
        }

        Ok(expr)
    }

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
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_prim(&mut self) -> Result<Expr, String> {
        match self.peek() {
            Some(c) if c.is_ascii_digit() => self.parse_const(),
            Some(c) if c.is_ascii_alphabetic() => {
                let var = self.parse_var()?;
                Ok(Expr::Var(var))
            }
            Some('(') => {
                self.next();
                let expr = self.parse_expr()?;
                self.expect(')')?;
                Ok(expr)
            }
            Some(c) => Err(format!("Token inesperado '{}'", c)),
            None => Err("Fim inesperado de entrada".into()),
        }
    }

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
    

    fn parse_kw(&mut self, kw: &str) -> Result<bool, String> {
        let mut clone = self.tokens.clone();
        for expected in kw.chars() {
            if Some(expected) != clone.next() {
                return Ok(false);
            }
        }

        // Remove or comment out the following check:
//      if let Some(next) = clone.peek() {
//          if next.is_ascii_alphanumeric() {
//              return Ok(false);
//          }
//      }

        self.tokens = clone;
        Ok(true)
    }
    
    fn match_kw(&mut self, kw: &str) -> Result<bool, String> {
        if self.parse_kw(kw)? {
            // Remove or comment out this loop:
            // for _ in 0..kw.len() {
            //     self.next();
            // }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn expect(&mut self, ch: char) -> Result<(), String> {
        match self.next() {
            Some(c) if c == ch => Ok(()),
            Some(c) => Err(format!("Esperado '{}', encontrado '{}'", ch, c)),
            None => Err(format!("Esperado '{}', mas fim de entrada", ch)),
        }
    }
}
