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

### 2. Run do projeto

```bash
cargo run
```

### 2. Run dos testes

```bash
cargo test
```