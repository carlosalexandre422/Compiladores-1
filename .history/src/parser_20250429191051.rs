use std::iter::{Filter, Peekable};
use std::str::Chars;

#[derive(Debug)]
pub enum Expr {
    Const(i32),
    OpBin {
        operador: char,
        op_esq: Box<Expr>,
        op_dir: Box<Expr>,
    },
}

pub struct Parser<'a> {
    tokens: Peekable<Filter<Chars<'a>, fn(&char) -> bool>>,
}

impl<'a> Parser<'a> {
    /// Constrói o parser removendo espaços.
    pub fn new(input: &'a str) -> Self {
        fn not_whitespace(c: &char) -> bool {
            !c.is_whitespace()
        }
        Self {
            tokens: input.chars().filter(not_whitespace as fn(&char) -> bool).peekable(),
        }
    }

    /// Inicia o parsing de uma expressão completa.
    pub fn parse(&mut self) -> Result<Expr, String> {
        self.parse_exp_a()
    }

    /// Analisa expressões de soma/subtração: <exp_a> ::= <exp_m> ((’+’ | ’-’) <exp_m>)*
    pub fn parse_exp_a(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_exp_m()?;

        while let Some(&op) = self.tokens.peek() {
            if op == '+' || op == '-' {
                self.tokens.next(); // Consome o operador
                let right = self.parse_exp_m()?;
                expr = Expr::OpBin {
                    operador: op,
                    op_esq: Box::new(expr),
                    op_dir: Box::new(right),
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    /// Analisa expressões de multiplicação/divisão: <exp_m> ::= <prim> ((’*’ | ’/’) <prim>)*
    pub fn parse_exp_m(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_prim()?;

        while let Some(&op) = self.tokens.peek() {
            if op == '*' || op == '/' {
                self.tokens.next(); // Consome o operador
                let right = self.parse_prim()?;
                expr = Expr::OpBin {
                    operador: op,
                    op_esq: Box::new(expr),
                    op_dir: Box::new(right),
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    /// Analisa números e expressões entre parênteses: <prim> ::= <num> | ’(’ <exp_a> ’)’
    pub fn parse_prim(&mut self) -> Result<Expr, String> {
        if let Some(&c) = self.tokens.peek() {
            if c.is_ascii_digit() {
                return self.parse_const();
            } else if c == '(' {
                self.tokens.next(); // Consome '('
                let expr = self.parse_exp_a();
                if self.tokens.next() != Some(')') {
                    return Err("Erro: ')' esperado".to_string());
                }
                return expr;
            } else {
                return Err(format!("Erro sintático: token inesperado '{}'", c));
            }
        }
        Err("Erro sintático: fim inesperado".to_string())
    }

    /// Analisa um número inteiro.
    pub fn parse_const(&mut self) -> Result<Expr, String> {
        let mut num = 0;
        while let Some(c) = self.tokens.peek().copied() {
            if c.is_ascii_digit() {
                num = num * 10 + c.to_digit(10).unwrap() as i32;
                self.tokens.next();
            } else {
                break;
            }
        }
        Ok(Expr::Const(num))
    }
}
