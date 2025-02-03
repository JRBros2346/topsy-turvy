FROM alpine:latest

# Install dependencies
RUN apk add --no-cache \
    rust \
    cargo \
    clang \
    llvm-dev \
    clang-dev \
    lld \
    libc-dev \
    gcc \
    g++ \
    make

# Set the working directory
WORKDIR /topsy-turvy

# Copy the Rust project
COPY . .

# Build the Rust application
RUN cargo build --release

# Expose the web app's port (modify if needed)
EXPOSE 3000

# Run the app
CMD ["./target/release/topsy-turvy"]
