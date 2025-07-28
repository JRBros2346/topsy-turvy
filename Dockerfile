FROM alpine:latest as builder

WORKDIR /app

RUN apk add --no-cache cargo

COPY Cargo.toml .
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo b --release && \
    rm -rf src 

COPY . .
RUN cargo t --release
RUN cargo r --release

FROM alpine:latest

COPY --from=builder /app/target/release/code-judge /usr/local/bin/code-judge

# Install minimal base build tools for MUSL + language runtimes
RUN apk add rust clang20 deno python3 openjdk21 nsjail

# Create app dir
WORKDIR /app

# Verify installed tools
RUN deno -V && \
    clang++ -v && \
    python3 -V && \
    javac -version && \
    java -version

EXPOSE 3000
CMD ["topsy-turvy"]
