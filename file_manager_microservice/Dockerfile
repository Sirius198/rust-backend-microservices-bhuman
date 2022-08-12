FROM rust:buster

ARG TOKEN=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

WORKDIR /workdir
COPY . .
RUN ls -a

WORKDIR /workdir/file_manager_microservice
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true

RUN git config --global url."https://api:$TOKEN@github.com/".insteadOf "https://github.com/"
RUN git config --global url."https://ssh:$TOKEN@github.com/".insteadOf "ssh://git@github.com/"
RUN git config --global url."https://git:$TOKEN@github.com/".insteadOf "git@github.com:"

RUN rustup component add rustfmt
RUN cargo build -r

EXPOSE 4000

CMD ["target/release/file-manager-microservice"]