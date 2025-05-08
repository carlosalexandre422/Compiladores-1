# 🛠️ Compilador Cmd — Rust + x86 Assembly

Este projeto é um **compilador simples** para a linguagem **Fun**, que interpreta programas escritos nessa linguagem, constrói uma árvore sintática (AST) e gera **código assembly x86-64**. O código assembly gerado pode ser compilado com `nasm` e `ld` para criar um executável.

## 📁 Estrutura do Projeto

- `main.rs` — Função principal do compilador.
- `parser.rs` — Parser recursivo descendente da linguagem Fun.
- `codegen.rs` — Gerador de código assembly.
- `texto.txt` — Arquivo de entrada com o código Fun.
- `output.asm` — Arquivo de saída com o código assembly gerado.
- `output.o` — Arquivo objeto gerado pelo `nasm` para criação do executável.
- `prog` — Arquivo executável gerado pelo `ld`.

---

## 🚀 Como Compilar e Executar

### 1. Build do projeto

```bash
cargo build
```

### 2. Escreva seu código Fun em **texto.txt**
Crie o arquivo texto.txt com um programa válido da linguagem Fun. Exemplo:

```bash
fun fib(n) {
  var res = 0;
  if n < 2 {
    res = 1;
  } else {
    res = fib(n - 1) + fib(n - 2);
  }
  return res;
}

main {
  return fib(6);
}
```

### 3. Execute o compilador

```bash
cargo run
```

O que irá acontecer:

- O compilador irá analisar sintaticamente o conteúdo de texto.txt.
- Gerará o código assembly correspondente em output.asm.
- Compilará o código assembly usando nasm e ld, criando o executável prog.

Você verá as mensagens:

```bash
Assembly gerado com sucesso em output.asm
Compilando o código assembly...
Executável gerado com sucesso: prog
```
Após isso, você poderá executar o programa com o comando:
```bash
./prog
```
## 🧪 Rodando os Testes
O projeto possui testes automatizados para validar o parser e o gerador de código.
#### Para executar os testes:

```bash
cargo test
```
Os testes estão implementados em parser.rs, codegen.rs e main.rs, dentro dos blocos #[cfg(test)].

## 🐧 Observações

Este projeto foi desenvolvido para ser executado em uma distribuição Linux.
Para gerar o executável prog, o compilador utiliza os seguintes comandos de sistema:

```bash
nasm -f elf64 output.asm -o output.o  # Gera o arquivo objeto
ld output.o -o prog                   # Gera o executável

```

## 👨‍💻 Desenvolvedores ##

- CARLOS ALEXANDRE SILVA DOS SANTOS - 20210025904
- JOAO VITOR TEIXEIRA BARRETO - 20210094349

