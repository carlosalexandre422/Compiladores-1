use crate::parser::{Expr, Cmd, Programa};
use std::collections::HashSet;

pub fn gerar_codigo(prog: &Programa) -> String {
    let mut codigo = String::new();
    let mut labels = 0;
    let mut variaveis: HashSet<String> = HashSet::new();

    // Seção de dados (.bss) para variáveis
    for (nome, _) in &prog.declaracoes {
        variaveis.insert(nome.clone());
    }

    codigo.push_str("section .bss\n");
    for var in &variaveis {
        codigo.push_str(&format!("{}: resq 1\n", var));
    }

    codigo.push_str("section .text\n");
    codigo.push_str("global _start\n\n");
    codigo.push_str("_start:\n");

    // Inicializações
    for (nome, expr) in &prog.declaracoes {
        codigo.push_str(&gerar_expr(expr, &mut labels));
        codigo.push_str(&format!("mov [{}], %rax\n", nome));
    }

    for cmd in &prog.comandos {
        codigo.push_str(&gerar_cmd(cmd, &mut labels, &variaveis));
    }

    // Código da expressão de retorno
    codigo.push_str(&gerar_expr(&prog.retorno, &mut labels));
    codigo.push_str("mov %rdi, %rax\n");
    codigo.push_str("mov $60, %rax\n"); // syscall exit
    codigo.push_str("syscall\n");

    codigo
}

fn gerar_expr(expr: &Expr, labels: &mut usize) -> String {
    match expr {
        Expr::Const(v) => format!("mov ${}, %rax\n", v),
        Expr::Var(nome) => format!("mov [{}], %rax\n", nome),
        Expr::OpBin { operador, esq, dir } => {
            let mut codigo = String::new();
            codigo.push_str(&gerar_expr(dir, labels));
            codigo.push_str("push %rax\n");
            codigo.push_str(&gerar_expr(esq, labels));
            codigo.push_str("pop %rbx\n");

            match operador.as_str() {
                "+" => codigo.push_str("add %rbx, %rax\n"),
                "-" => codigo.push_str("sub %rbx, %rax\n"),
                "*" => codigo.push_str("imul %rbx, %rax\n"),
                "/" => codigo.push_str("cqo\nidiv %rbx\n"),
                "==" => {
                    codigo.push_str("xor %rcx, %rcx\n");
                    codigo.push_str("cmp %rax, %rbx\n");
                    codigo.push_str("setz %cl\n");
                    codigo.push_str("mov %rcx, %rax\n");
                }
                "<" => {
                    codigo.push_str("xor %rcx, %rcx\n");
                    codigo.push_str("cmp %rax, %rbx\n");
                    codigo.push_str("setl %cl\n");
                    codigo.push_str("mov %rcx, %rax\n");
                }
                ">" => {
                    codigo.push_str("xor %rcx, %rcx\n");
                    codigo.push_str("cmp %rax, %rbx\n");
                    codigo.push_str("setg %cl\n");
                    codigo.push_str("mov %rcx, %rax\n");
                }
                _ => panic!("Operador inválido: {}", operador),
            }

            codigo
        }
    }
}

fn gerar_cmd(cmd: &Cmd, labels: &mut usize, vars: &HashSet<String>) -> String {
    match cmd {
        Cmd::Atrib { nome, expr } => {
            if !vars.contains(nome) {
                panic!("Variável não declarada: {}", nome);
            }
            let mut codigo = gerar_expr(expr, labels);
            codigo.push_str(&format!("mov [{}], %rax\n", nome));
            codigo
        }

        Cmd::If { cond, then_cmds, else_cmds } => {
            let l_falso = *labels;
            *labels += 1;
            let l_fim = *labels;
            *labels += 1;

            let mut codigo = gerar_expr(cond, labels);
            codigo.push_str("cmp $0, %rax\n");
            codigo.push_str(&format!("jz Lfalso{}\n", l_falso));

            for c in then_cmds {
                codigo.push_str(&gerar_cmd(c, labels, vars));
            }
            codigo.push_str(&format!("jmp Lfim{}\n", l_fim));
            codigo.push_str(&format!("Lfalso{}:\n", l_falso));

            for c in else_cmds {
                codigo.push_str(&gerar_cmd(c, labels, vars));
            }
            codigo.push_str(&format!("Lfim{}:\n", l_fim));
            codigo
        }

        Cmd::While { cond, body } => {
            let l_ini = *labels;
            *labels += 1;
            let l_fim = *labels;
            *labels += 1;

            let mut codigo = format!("Linicio{}:\n", l_ini);
            codigo.push_str(&gerar_expr(cond, labels));
            codigo.push_str("cmp $0, %rax\n");
            codigo.push_str(&format!("jz Lfim{}\n", l_fim));

            for c in body {
                codigo.push_str(&gerar_cmd(c, labels, vars));
            }

            codigo.push_str(&format!("jmp Linicio{}\n", l_ini));
            codigo.push_str(&format!("Lfim{}:\n", l_fim));
            codigo
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    fn gerar_ok(source: &str) {
        let mut parser = Parser::new(source);
        let programa = parser.parse_programa().expect("Parsing falhou");
        let codigo = gerar_codigo(&programa);
        assert!(codigo.contains("mov"), "Código assembly gerado incorretamente:\n{}", codigo);
    }

    #[test]
    fn test_codegen_if() {
        gerar_ok("x = 1; { if x == 1 { x = 2; } else { x = 3; } return x; }");
    }

    #[test]
    fn test_codegen_while() {
        gerar_ok("i = 0; { while i < 3 { i = i + 1; } return i; }");
    }

    #[test]
    fn test_codegen_expressao_simples() {
        gerar_ok("a = 10; b = 5; c = 0; { c = a / b; return c; }");
    }
}
