//CARLOS ALEXANDRE SILVA DOS SANTOS - 20210025904
//JOAO VITOR TEIXEIRA BARRETO - 20210094349

use std::fs::File;
use std::io::{self, Read, Write};
use parser::{Parser, Expr};
use std::process;

mod parser;

/// Gera código assembly simples para a expressão.
fn gerar_codigo(expr: &Expr) -> String {
    match expr {
        Expr::Const(v) => format!("mov ${}, %rax\n", v),
        Expr::OpBin { operador, op_esq, op_dir } => {
            let mut codigo = String::new();
            // Gera código da subexpressão direita, põe em %rax.
            codigo.push_str(&gerar_codigo(op_dir));
            // Empilha %rax.
            codigo.push_str("push %rax\n");
            // Gera código da subexpressão esquerda, põe em %rax.
            codigo.push_str(&gerar_codigo(op_esq));
            // Remove a subexpr direita da pilha e põe em %rbx.
            codigo.push_str("pop %rbx\n");
            // Aplica o operador.
            match operador {
                '+' => codigo.push_str("add %rbx, %rax\n"),
                '-' => codigo.push_str("sub %rbx, %rax\n"),
                '*' => codigo.push_str("imul %rbx, %rax\n"),
                '/' => codigo.push_str("cqo\nidiv %rbx\n"),
                _ => panic!("Operador inválido\n"),
            }
            codigo
        }
    }
}

/// Lê o conteúdo de um arquivo texto.
fn read_file(filename: &str) -> io::Result<String> {
    let mut file = File::open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

/// Função principal: faz checagens, gera a AST e então gera o asm em "output.asm".
fn main() -> io::Result<()> {
    let filename = "texto.txt";
    let input = read_file(filename).unwrap_or_else(|_| {
        eprintln!("Erro ao abrir o arquivo");
        process::exit(1);
    });

    // Cria o parser.
    let mut parser = Parser::new(&input);

    // Tenta analisar (parse) a expressão.
    match parser.parse() {
        Ok(expr) => {
            // Gera o código assembly.
            let assembly = gerar_codigo(&expr);
            // Cria um arquivo e salva o asm.
            let mut file = File::create("output.asm")?;
            file.write_all(assembly.as_bytes())?;
            println!("Código assembly gerado em output.asm");
        }
        Err(err) => eprintln!("Erro: {}", err),
    }
    
    Ok(())
}