// Importa os tipos definidos no módulo de parser
use crate::parser::{Expr, Cmd, Programa, FunDecl};
// Importa o HashMap da biblioteca padrão
use std::collections::HashMap;

// Função principal que gera o código assembly a partir de um programa
pub fn gerar_codigo(prog: &Programa) -> String {
    let mut codigo = String::new(); // Código final a ser construído
    let mut labels = 0; // Contador de labels para gerar nomes únicos

    // Declaração da seção BSS para alocar espaço para variáveis globais
    codigo.push_str("section .bss\n");
    for (nome, _) in &prog.globais {
        codigo.push_str(&format!("{}: resq 1\n", nome)); // Reserva 8 bytes (1 quadword)
    }

    // Início da seção de código executável
    codigo.push_str("section .text\n");
    codigo.push_str("global _start\n"); // Ponto de entrada do programa

    // Gera o código das funções definidas no programa
    for func in &prog.funcoes {
        codigo.push_str(&gerar_funcao(func, &mut labels));
    }

    // Gera o código do bloco principal (iniciado por _start)
    codigo.push_str("\n_start:\n");

    // Inicializa as variáveis globais com as expressões correspondentes
    for (nome, expr) in &prog.globais {
        codigo.push_str(&gerar_expr(expr, &mut labels, &HashMap::new()));
        codigo.push_str(&format!("mov [{}], rax\n", nome)); // Armazena em memória
    }

    // Gera os comandos principais do programa
    for cmd in &prog.principal {
        codigo.push_str(&gerar_cmd(cmd, &mut labels, &HashMap::new()));
    }

    // Gera o valor de retorno do programa
    codigo.push_str(&gerar_expr(&prog.retorno, &mut labels, &HashMap::new()));
    codigo.push_str("mov rdi, rax\n"); // Passa retorno como argumento do exit
    codigo.push_str("mov rax, 60\n");  // syscall number de exit
    codigo.push_str("syscall\n");      // chama o kernel

    codigo
}

// Gera o código de uma função
fn gerar_funcao(func: &FunDecl, labels: &mut usize) -> String {
    let mut codigo = String::new();
    let mut deslocamentos = HashMap::new();
    let mut offset = 16; // Parâmetros começam depois do rbp

    // Atribui offset para parâmetros (a partir de rbp+16, rbp+24, etc.)
    for param in &func.parametros {
        deslocamentos.insert(param.clone(), offset);
        offset += 8;
    }

    // Atribui offset negativo para variáveis locais (rbp-8, rbp-16, etc.)
    let mut var_offset = -8;
    for (nome, _) in &func.variaveis {
        deslocamentos.insert(nome.clone(), var_offset);
        var_offset -= 8;
    }

    // Cabeçalho da função
    codigo.push_str(&format!("\n{}:\n", func.nome));
    codigo.push_str("push rbp\n");
    codigo.push_str("mov rbp, rsp\n");

    // Aloca espaço para variáveis locais
    let tamanho_stack = func.variaveis.len() * 8;
    if tamanho_stack > 0 {
        codigo.push_str(&format!("sub rsp, {}\n", tamanho_stack));
    }

    // Inicializa variáveis locais
    for (nome, expr) in &func.variaveis {
        codigo.push_str(&gerar_expr(expr, labels, &deslocamentos));
        if let Some(offset) = deslocamentos.get(nome) {
            let sinal = if *offset < 0 { "-" } else { "+" };
            codigo.push_str(&format!("mov [rbp{}{}], rax\n", sinal, offset.abs()));
        }
    }

    // Gera os comandos da função
    for cmd in &func.comandos {
        codigo.push_str(&gerar_cmd(cmd, labels, &deslocamentos));
    }

    // Gera o valor de retorno da função
    codigo.push_str(&gerar_expr(&func.retorno, labels, &deslocamentos));

    // Libera espaço de pilha
    if tamanho_stack > 0 {
        codigo.push_str(&format!("add rsp, {}\n", tamanho_stack));
    }

    // Epílogo da função
    codigo.push_str("pop rbp\n");
    codigo.push_str("ret\n");

    codigo
}

// Gera código para uma expressão
fn gerar_expr(expr: &Expr, labels: &mut usize, deslocamentos: &HashMap<String, isize>) -> String {
    match expr {
        // Expressão constante: move o valor para rax
        Expr::Const(v) => format!("mov rax, {}\n", v),

        // Variável: local (pilha) ou global (memória)
        Expr::Var(nome) => {
            if let Some(offset) = deslocamentos.get(nome) {
                let sinal = if *offset < 0 { "-" } else { "+" };
                format!("mov rax, [rbp{}{}]\n", sinal, offset.abs())
            } else {
                format!("mov rax, [{}]\n", nome)
            }
        }

        // Operação binária
        Expr::OpBin { operador, esq, dir } => {
            let mut codigo = String::new();
            codigo.push_str(&gerar_expr(dir, labels, deslocamentos)); // Avalia direita primeiro
            codigo.push_str("push rax\n"); // Salva resultado
            codigo.push_str(&gerar_expr(esq, labels, deslocamentos)); // Avalia esquerda
            codigo.push_str("pop rbx\n"); // Recupera direita em rbx

            match operador.as_str() {
                "+" => codigo.push_str("add rax, rbx\n"),
                "-" => codigo.push_str("sub rax, rbx\n"),
                "*" => codigo.push_str("imul rax, rbx\n"),
                "/" => {
                    codigo.push_str("cqo\n"); // Estende rax para rdx:rax
                    codigo.push_str("idiv rbx\n"); // Divide rdx:rax por rbx, resultado em rax
                }
                "==" => {
                    codigo.push_str("xor rcx, rcx\n");
                    codigo.push_str("cmp rax, rbx\n");
                    codigo.push_str("setz cl\n");
                    codigo.push_str("mov rax, rcx\n");
                }
                "<" => {
                    codigo.push_str("xor rcx, rcx\n");
                    codigo.push_str("cmp rax, rbx\n");
                    codigo.push_str("setl cl\n");
                    codigo.push_str("mov rax, rcx\n");
                }
                ">" => {
                    codigo.push_str("xor rcx, rcx\n");
                    codigo.push_str("cmp rax, rbx\n");
                    codigo.push_str("setg cl\n");
                    codigo.push_str("mov rax, rcx\n");
                }
                _ => panic!("Operador inválido: {}", operador),
            }

            codigo
        }

        // Chamada de função
        Expr::Call { nome, args } => {
            let mut codigo = String::new();
            for arg in args.iter().rev() {
                codigo.push_str(&gerar_expr(arg, labels, deslocamentos));
                codigo.push_str("push rax\n"); // Empilha argumentos
            }
            codigo.push_str(&format!("call {}\n", nome)); // Chama a função
            if !args.is_empty() {
                codigo.push_str(&format!("add rsp, {}\n", args.len() * 8)); // Desempilha
            }
            codigo
        }
    }
}

// Gera código para um comando
fn gerar_cmd(cmd: &Cmd, labels: &mut usize, deslocamentos: &HashMap<String, isize>) -> String {
    match cmd {
        // Atribuição de valor a uma variável
        Cmd::Atrib { nome, expr } => {
            let mut codigo = gerar_expr(expr, labels, deslocamentos);
            if let Some(offset) = deslocamentos.get(nome) {
                let sinal = if *offset < 0 { "-" } else { "+" };
                codigo.push_str(&format!("mov [rbp{}{}], rax\n", sinal, offset.abs()));
            } else {
                codigo.push_str(&format!("mov [{}], rax\n", nome));
            }
            codigo
        }

        // Comando if com else
        Cmd::If { cond, then_cmds, else_cmds } => {
            let l_falso = *labels;
            *labels += 1;
            let l_fim = *labels;
            *labels += 1;

            let mut codigo = gerar_expr(cond, labels, deslocamentos);
            codigo.push_str("cmp rax, 0\n");
            codigo.push_str(&format!("je Lfalso{}\n", l_falso));

            for c in then_cmds {
                codigo.push_str(&gerar_cmd(c, labels, deslocamentos));
            }

            codigo.push_str(&format!("jmp Lfim{}\n", l_fim));
            codigo.push_str(&format!("Lfalso{}:\n", l_falso));

            for c in else_cmds {
                codigo.push_str(&gerar_cmd(c, labels, deslocamentos));
            }

            codigo.push_str(&format!("Lfim{}:\n", l_fim));
            codigo
        }

        // Laço while
        Cmd::While { cond, body } => {
            let l_ini = *labels;
            *labels += 1;
            let l_fim = *labels;
            *labels += 1;

            let mut codigo = format!("Linicio{}:\n", l_ini);
            codigo.push_str(&gerar_expr(cond, labels, deslocamentos));
            codigo.push_str("cmp rax, 0\n");
            codigo.push_str(&format!("je Lfim{}\n", l_fim));

            for c in body {
                codigo.push_str(&gerar_cmd(c, labels, deslocamentos));
            }

            codigo.push_str(&format!("jmp Linicio{}\n", l_ini));
            codigo.push_str(&format!("Lfim{}:\n", l_fim));
            codigo
        }
    }
}

//////////////
/// TESTES ///
//////////////
#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{Expr, Cmd, Programa};
    use std::collections::HashMap;

    // Testa a expressão constante (um valor fixo)
    #[test]
    fn test_expr_const() {
        let codigo = gerar_expr(&Expr::Const(42), &mut 0, &HashMap::new());
        // Verifica se o código gerado move o valor constante para o registrador rax
        assert_eq!(codigo.trim(), "mov rax, 42");
    }

    // Testa a expressão com uma variável global
    #[test]
    fn test_expr_var_global() {
        let codigo = gerar_expr(&Expr::Var("x".to_string()), &mut 0, &HashMap::new());
        // Verifica se o código gerado acessa a variável global "x"
        assert_eq!(codigo.trim(), "mov rax, [x]");
    }

    // Testa a expressão com uma variável local (com offset)
    #[test]
    fn test_expr_var_local() {
        let mut mapa = HashMap::new();
        mapa.insert("x".to_string(), -8); // "x" está no endereço [rbp-8]
        let codigo = gerar_expr(&Expr::Var("x".to_string()), &mut 0, &mapa);
        // Verifica se o código gerado acessa a variável local "x" com o offset correto
        assert_eq!(codigo.trim(), "mov rax, [rbp-8]");
    }

    // Testa a expressão com uma operação binária de adição
    #[test]
    fn test_expr_opbin_add() {
        let expr = Expr::OpBin {
            operador: "+".to_string(),
            esq: Box::new(Expr::Const(2)),
            dir: Box::new(Expr::Const(3)),
        };
        let codigo = gerar_expr(&expr, &mut 0, &HashMap::new());
        // Verifica se o código gerado realiza a operação de adição
        assert!(codigo.contains("add rax, rbx"));
    }

    // Testa o comando de atribuição de uma constante em uma variável global
    #[test]
    fn test_cmd_atrib_global() {
        let cmd = Cmd::Atrib {
            nome: "x".to_string(),
            expr: Expr::Const(5),
        };
        let codigo = gerar_cmd(&cmd, &mut 0, &HashMap::new());
        // Verifica se o código gerado atribui o valor de rax à variável global "x"
        assert!(codigo.contains("mov [x], rax"));
    }

    // Testa a geração de código de um programa mínimo
    #[test]
    fn test_gerar_codigo_minimal() {
        let prog = Programa {
            globais: vec![("x".to_string(), Expr::Const(1))],
            funcoes: vec![], // Sem funções definidas
            principal: vec![Cmd::Atrib {
                nome: "x".to_string(),
                expr: Expr::Const(2),
            }],
            retorno: Expr::Var("x".to_string()),
        };
        let codigo = gerar_codigo(&prog);
        // Verifica se o código gerado contém o início correto do programa e a atribuição para "x"
        assert!(codigo.contains("_start:"));
        assert!(codigo.contains("mov [x], rax"));
        assert!(codigo.contains("mov rdi, rax")); // Retorna o valor de "x"
    }
}
