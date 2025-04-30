use std::iter::Peekable;
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
    tokens: Peekable<Chars<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        // no longer filter out ALL whitespace here
        Parser {
            tokens: input.chars().peekable(),
        }
    }

    fn next(&mut self) -> Option<char> {
        // skip *leading* whitespace
        while let Some(&c) = self.tokens.peek() {
            if c.is_whitespace() {
                self.tokens.next();
            } else {
                break;
            }
        }
        self.tokens.next()
    }

    fn peek(&mut self) -> Option<char> {
        // look ahead, skipping whitespace
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

    pub fn parse_programa(&mut self) -> Result<Programa, String> {
        let mut decls = Vec::new();

        // parse top‐level declarations: ident = expr ;
        while let Some(c) = self.peek() {
            if !c.is_ascii_alphabetic() {
                break;
            }
            // check if next keyword is "if" or "while"
            let mut clone = self.tokens.clone();
            while let Some(&ch) = clone.peek() {
                if ch.is_whitespace() {
                    clone.next();
                } else {
                    break;
                }
            }
            // if ...
            if clone.next() == Some('i') &&
               clone.next() == Some('f') &&
               !matches!(clone.peek(), Some(x) if x.is_ascii_alphanumeric())
            {
                break;
            }
            // while ...
            let mut clone2 = self.tokens.clone();
            while let Some(&ch) = clone2.peek() {
                if ch.is_whitespace() {
                    clone2.next();
                } else {
                    break;
                }
            }
            if clone2.next() == Some('w') &&
               clone2.next() == Some('h') &&
               clone2.next() == Some('i') &&
               clone2.next() == Some('l') &&
               clone2.next() == Some('e') &&
               !matches!(clone2.peek(), Some(x) if x.is_ascii_alphanumeric())
            {
                break;
            }

            // it's a declaration
            let var = self.parse_var()?;
            self.expect('=')?;
            let expr = self.parse_expr()?;
            self.expect(';')?;
            decls.push((var, expr));
        }

        // parse block of commands
        self.expect('{')?;
        let mut cmds = Vec::new();
        while self.peek() != Some('r') {
            cmds.push(self.parse_cmd()?);
        }

        // return expression
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
        } else if self.match_kw("while")? {
            let cond = self.parse_expr()?;
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
        let mut expr = self.parse_exp_a()?;

        // comparison operators: ==, <, >
        while let Some(c) = self.peek() {
            let op = match c {
                '=' => {
                    self.next();
                    if self.next() == Some('=') {
                        "==".to_string()
                    } else {
                        return Err("Operador = mal formado".into());
                    }
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
        // skip whitespace
        while let Some(&c) = clone.peek() {
            if c.is_whitespace() {
                clone.next();
            } else {
                break;
            }
        }
        // match keyword
        for expected in kw.chars() {
            if Some(expected) != clone.next() {
                return Ok(false);
            }
        }
        // ensure not prefix of ident
        if let Some(&next) = clone.peek() {
            if next.is_ascii_alphanumeric() {
                return Ok(false);
            }
        }
        // commit
        self.tokens = clone;
        Ok(true)
    }

    fn match_kw(&mut self, kw: &str) -> Result<bool, String> {
        if self.parse_kw(kw)? {
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