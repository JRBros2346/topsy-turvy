FROM alpine:latest

# Install dependencies
RUN apk add --no-cache cargo clang20 python3 openjdk21 deno

# Set the working directory
WORKDIR /topsy-turvy


# Test dependencies
RUN rustc -V
RUN python3 -v
RUN javac -version
RUN java -version
RUN deno -v
RUN clang++ -v

COPY Cargo.toml ./
RUN mkdir src
RUN echo "fn main() {}" > src/main.rs
RUN cargo build --release


# Copy the Rust project
COPY . .

# Build the Rust application
RUN cargo build --release

# Expose the web app's port (modify if needed)
EXPOSE 3000

# Run the app
CMD ["./target/release/topsy-turvy"]
