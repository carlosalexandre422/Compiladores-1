// CARLOS ALEXANDRE SILVA DOS SANTOS - 20210025904
// JOAO VITOR TEIXEIRA BARRETO - 20210094349
use std::fs::File;
use std::io::{self, Read, Write};
use std::process::{self, Command};

mod parser;
mod codegen;

use parser::Parser;

/// Lê o conteúdo de um arquivo e o retorna como `String`.
fn read_file(filename: &str) -> io::Result<String> {
    let mut file = File::open(filename)?; // Corrigido para usar `filename` como argumento
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

/// Função para rodar um comando no sistema
fn run_command(command: &str, args: &[&str]) -> process::ExitStatus {
    let status = Command::new(command)
        .args(args)
        .status()
        .expect("Falha ao executar o comando");

    if !status.success() {
        eprintln!("Erro ao executar o comando: {} {:?}", command, args);
        process::exit(1);
    }
    status
}

fn main() {
    // Lê o conteúdo do arquivo de entrada
    let input = read_file("texto.txt").unwrap_or_else(|_| {
        eprintln!("Erro ao abrir o arquivo");
        process::exit(1);
    });

    // Cria o parser e tenta processar o programa
    let mut parser = Parser::new(&input);
    match parser.parse_programa() {
        Ok(prog) => {
            // Gera o código assembly a partir da estrutura do programa
            let codigo = codegen::gerar_codigo(&prog);

            // Escreve o código assembly gerado em "output.asm"
            let mut file = File::create("output.asm").expect("Erro ao criar output.asm");
            file.write_all(codigo.as_bytes()).expect("Erro ao escrever no arquivo");

            println!("Assembly gerado com sucesso em output.asm");

            // Executa os comandos para compilar o código assembly
            println!("Compilando o código assembly...");
            run_command("nasm", &["-f", "elf64", "output.asm", "-o", "output.o"]);
            run_command("ld", &["output.o", "-o", "prog"]);

            println!("Executável gerado com sucesso: prog");
        }
        Err(err) => {
            // Exibe erro de parsing se houver
            eprintln!("Erro de parsing: {}", err);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;

    /// Testa a função `read_file` lendo um arquivo temporário
    #[test]
    fn test_read_file() {
        let test_filename = "test_input.txt";
        let content = "var x = 10;";
        let mut file = File::create(test_filename).unwrap();
        file.write_all(content.as_bytes()).unwrap();

        let read = read_file(test_filename).unwrap();
        assert_eq!(read, content);

        fs::remove_file(test_filename).unwrap();
    }

    /// Testa o parser com um código de programa mínimo
    #[test]
    fn test_parser_programa_minimo() {
        let code = r#"
            main {
                return 42;
            }
        "#;
        let mut parser = Parser::new(code);
        let result = parser.parse_programa();
        assert!(result.is_ok(), "Parsing falhou: {:?}", result.err());
    }

    /// Testa erro de parsing com input malformado
    #[test]
    fn test_parser_erro() {
        let code = r#"
            main {
                return ;
            }
        "#;
        let mut parser = Parser::new(code);
        let result = parser.parse_programa();
        assert!(result.is_err(), "Era esperado erro de parsing");
    }
}
