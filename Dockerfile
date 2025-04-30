# Usar a imagem oficial do Rust
FROM rust:latest

# Definir diretório de trabalho
WORKDIR /app

# Copiar os arquivos do projeto para dentro do contêiner
COPY . .

# Compilar o projeto
RUN cargo build --release

# Definir o comando padrão para rodar o binário
CMD ["./target/release/compiler"]
