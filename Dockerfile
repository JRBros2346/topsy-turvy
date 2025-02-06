FROM alpine:latest

# Install dependencies
RUN apk add --no-cache cargo clang19 python3 openjdk21 deno

# Set the working directory
WORKDIR /topsy-turvy

# Copy the Rust project
COPY . .

# Test dependencies
RUN rustc -V
RUN clang++ -v
RUN python3 -v
RUN javac -version
RUN java -version
RUN deno -v

# Build the Rust application
RUN cargo build --release

# Expose the web app's port (modify if needed)
EXPOSE 3000

# Run the app
CMD ["./target/release/topsy-turvy"]
