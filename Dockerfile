FROM rust:1.65

ARG PROJECTNAME=server-cli

WORKDIR /code
RUN cargo init
RUN echo "rand = \"0.8\"" >> Cargo.toml
RUN cargo fetch

WORKDIR /code
Copy . .

EXPOSE 27008/udp

RUN cargo install --path server-cli

ENV RUST_LOG=INFO

CMD ["server-cli"]
