FROM alpine:latest

# Install build dependencies
RUN apk add --no-cache \
    build-base \
    cargo \
    rust \
    deno \
    clang20 \
    python3 \
    openjdk21 \
    nsjail

# Create and set working directory
WORKDIR /topsy-turvy

# Copy only Cargo.toml first to leverage Docker layer caching
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    touch src/lib.rs && \
    cargo b --release && \
    rm -rf src

# Copy the full source now
COPY . .

# Optionally run tests here only in dev builds
RUN cargo t --release

# Build & install the binary
RUN cargo b --release && \
    mv target/release/topsy-turvy /usr/local/bin/ && \
    strip /usr/local/bin/topsy-turvy

# Clean up build tools and files
RUN apk del cargo build-base && \
    rm -rf /topsy-turvy /root/.cargo /root/.rustup /usr/lib/rustlib

# Verify installed tools (combine into single layer)
RUN rustc -V && \
    deno -v && \
    clang++ -v && \
    python3 -V && \
    javac -version && \
    java -version

# Expose port if needed
EXPOSE 3000

# Entry point
CMD ["topsy-turvy"]
