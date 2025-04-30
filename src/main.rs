//CARLOS ALEXANDRE SILVA DOS SANTOS - 20210025904
//JOAO VITOR TEIXEIRA BARRETO - 20210094349

use std::fs::File;
use std::io::{self, Read, Write};
use std::process;

// Módulos locais: parser (analisador sintático) e codegen (gerador de código)
mod parser;
mod codegen;
use parser::Parser; // Importa a struct Parser

/// Lê o conteúdo de um arquivo e retorna como String
/// Caso ocorra erro (ex: arquivo não existe), ele será propagado com "?'
fn read_file(filename: &str) -> io::Result<String> {
    let mut file = File::open(filename)?;        // Abre o arquivo
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;         // Lê tudo para a string
    Ok(contents)                                 // Retorna o conteudo
}

/// Função principal: executa o compilador
fn main() {
    // Tenta abrir e ler o arquivo de entrada
    let input = read_file("texto.txt").unwrap_or_else(|_| {
        eprintln!("Erro ao abrir o arquivo");    
        process::exit(1);                       
    });

    // Cria um novo parser com o conteúdo lido
    let mut parser = Parser::new(&input);


    // Analisa a estrutura do programa com base na gramática Cmd
    match parser.parse_programa() {
        Ok(prog) => {
            // Geração de código assembly a partir da AST
            let codigo = codegen::gerar_codigo(&prog);

            // Salva o código gerado no arquivo de saída
            let mut file = File::create("output.asm").expect("Erro ao criar output.asm");
            file.write_all(codigo.as_bytes()).expect("Erro ao escrever o arquivo");

            println!("Assembly gerado com sucesso em output.asm");
        }
        // erro
        Err(err) => eprintln!("Erro de parsing: {}", err),
    }
}
