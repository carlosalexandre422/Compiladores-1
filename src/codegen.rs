use crate::parser::{Expr, Cmd, Programa};
use std::collections::HashSet;

/// Função principal que gera código assembly a partir de um programa Cmd
pub fn gerar_codigo(prog: &Programa) -> String {
    let mut codigo = String::new();        // Acumulador de código assembly
    let mut labels = 0;                    // Contador para gerar rótulos únicos
    let mut variaveis: HashSet<String> = HashSet::new(); // Variáveis declaradas

    // Coleta as variáveis declaradas para alocar no .bss
    for (nome, _) in &prog.declaracoes {
        variaveis.insert(nome.clone());
    }

    // Geração da secao .bss (var globais nao inicializadas)
    codigo.push_str("section .bss\n");
    for var in &variaveis {
        codigo.push_str(&format!("{}: resq 1\n", var));
    }

    // Início da seção de texto do programa
    codigo.push_str("section .text\n");
    codigo.push_str("global _start\n\n");
    codigo.push_str("_start:\n");

    // Gera código para as declarações (inicializaçoes)
    for (nome, expr) in &prog.declaracoes {
        codigo.push_str(&gerar_expr(expr, &mut labels));
        codigo.push_str(&format!("mov [{}], %rax\n", nome));
    }

    // Gera código para cada comando (atribuições, ifs, loops)
    for cmd in &prog.comandos {
        codigo.push_str(&gerar_cmd(cmd, &mut labels, &variaveis));
    }

    // Gera código da expressão de retorno final
    codigo.push_str(&gerar_expr(&prog.retorno, &mut labels));
    codigo.push_str("mov %rdi, %rax\n");   
    codigo.push_str("mov $60, %rax\n");    // syscall exit
    codigo.push_str("syscall\n");

    codigo
}

/// Geração de código para uma expressão (Expr)
fn gerar_expr(expr: &Expr, labels: &mut usize) -> String {
    match expr {
        // Literal: carrega diretamente para %rax
        Expr::Const(v) => format!("mov ${}, %rax\n", v),

        // Variável: carrega valor da memória para %rax
        Expr::Var(nome) => format!("mov [{}], %rax\n", nome),

        // Operação binária
        Expr::OpBin { operador, esq, dir } => {
            let mut codigo = String::new();
            // Gera código da direita e empilha
            codigo.push_str(&gerar_expr(dir, labels));
            codigo.push_str("push %rax\n");
            // Gera código da esquerda
            codigo.push_str(&gerar_expr(esq, labels));
            codigo.push_str("pop %rbx\n"); // Recupera direita em %rbx

            // Aplica o operador
            match operador.as_str() {
                "+" => codigo.push_str("add %rbx, %rax\n"),
                "-" => codigo.push_str("sub %rbx, %rax\n"),
                "*" => codigo.push_str("imul %rbx, %rax\n"),
                "/" => codigo.push_str("cqo\nidiv %rbx\n"), // divide rax por rbx

                // Comparação de igualdade: setz → 1 se igual
                "==" => {
                    codigo.push_str("xor %rcx, %rcx\n");
                    codigo.push_str("cmp %rax, %rbx\n");
                    codigo.push_str("setz %cl\n");
                    codigo.push_str("mov %rcx, %rax\n");
                }
                // Menor que
                "<" => {
                    codigo.push_str("xor %rcx, %rcx\n");
                    codigo.push_str("cmp %rax, %rbx\n");
                    codigo.push_str("setl %cl\n");
                    codigo.push_str("mov %rcx, %rax\n");
                }
                // Maior que
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

/// Geração de código para comandos (Cmd)
fn gerar_cmd(cmd: &Cmd, labels: &mut usize, vars: &HashSet<String>) -> String {
    match cmd {
        // Atribuição: expr → %rax, depois salva em variável
        Cmd::Atrib { nome, expr } => {
            if !vars.contains(nome) {
                panic!("Variável não declarada: {}", nome);
            }
            let mut codigo = gerar_expr(expr, labels);
            codigo.push_str(&format!("mov [{}], %rax\n", nome));
            codigo
        }

        // Comando if: gera condicional com saltos
        Cmd::If { cond, then_cmds, else_cmds } => {
            let l_falso = *labels;
            *labels += 1;
            let l_fim = *labels;
            *labels += 1;

            let mut codigo = gerar_expr(cond, labels);
            codigo.push_str("cmp $0, %rax\n"); // Se cond == 0, salta para else
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

        // Comando while: cria loop com verificação no início
        Cmd::While { cond, body } => {
            let l_ini = *labels;
            *labels += 1;
            let l_fim = *labels;
            *labels += 1;

            let mut codigo = format!("Linicio{}:\n", l_ini);
            codigo.push_str(&gerar_expr(cond, labels));
            codigo.push_str("cmp $0, %rax\n"); // Se falso, sai do loop
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

///////////////////////////////////////
/// TESTES DE GERAÇÃO DE CÓDIGO ASM ///
///////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    /// Utilitário de teste: parseia o código-fonte e gera código assembly
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
