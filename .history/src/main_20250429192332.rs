use std::fs::File;
use std::io::{self, Read};
use std::process;

mod parser;
use parser::{Parser, Programa};

mod codegen;

fn read_file(filename: &str) -> io::Result<String> {
    let mut file = File::open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn main() {
    let input = read_file("texto.txt").unwrap_or_else(|_| {
        eprintln!("Erro ao abrir o arquivo");
        process::exit(1);
    });

    let mut parser = Parser::new(&input);

    match parser.parse_programa() {
        Ok(prog) => {
            let codigo = codegen::gerar_codigo(&prog);
            let mut file = File::create("output.asm").expect("Erro ao criar output.asm");
            file.write_all(codigo.as_bytes()).expect("Erro ao escrever o arquivo");
            println!("Assembly gerado com sucesso em output.asm");
        }
        Err(err) => eprintln!("Erro de parsing: {}", err),
    }
    
}
