FROM alpine:latest

# Install dependencies
RUN apk add --no-cache cargo clang
# firejail

# Set the working directory
WORKDIR /topsy-turvy

# Copy the Rust project
COPY . .

# Test dependencies
RUN rustc -V
RUN clang++ -v
# RUN timeout --version

# Build the Rust application
RUN cargo build --release

# Expose the web app's port (modify if needed)
EXPOSE 3000

# Run the app
CMD ["./target/release/topsy-turvy"]
