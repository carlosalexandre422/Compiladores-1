# 🛠️ Compilador Cmd — Rust + x86 Assembly

Este projeto é um **compilador simples** que interpreta programas escritos na linguagem **Cmd**, constrói uma árvore sintática (AST) e gera **código assembly x86-64**, compilável com `nasm` e `ld`.

---

## 📁 Estrutura do Projeto

- `main.rs` — Função principal do compilador.
- `parser.rs` — Parser recursivo descendente da linguagem Cmd.
- `codegen.rs` — Gerador de código assembly.
- `texto.txt` — Arquivo de entrada com o código Cmd.
- `output.asm` — Arquivo de saída com o código assembly gerado.

---

## 🚀 Como Compilar e Executar

### 1. Build do projeto

```bash
cargo build
```

### 2. Escreva seu código Cmd em **texto.txt**
Crie o arquivo texto.txt com um programa válido da linguagem Cmd. Exemplo:

```bash
a = 4;
{
    if a < 10 {
        a = 1;
    } else {
        a = 0;
    }
    return a;
}
```

### 3. Execute o compilador

```bash
cargo run
```

Isso irá:

- Analisar sintaticamente o conteúdo de texto.txt
- Gerar o código assembly correspondente
- Salvar o resultado no arquivo output.asm

Você verá a mensagem:

```bash
Assembly gerado com sucesso em output.asm
```

### 🧪 Rodando os Testes
O projeto possui testes automatizados para validar o parser e o gerador de código.
#### Para executar os testes:

```bash
cargo test
```
Os testes estão implementados em parser.rs e codegen.rs, dentro dos blocos #[cfg(test)].
