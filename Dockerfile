# Use a multi-architecture compatible Rust image
FROM rust as builder

# Set up the working directory
WORKDIR /postmyrustache

# Copy the source code into the container
COPY . .

# Build the application
RUN cargo build --release

# Create a new stage with a minimal image
FROM scratch

# Copy the binary to the new stage
COPY --from=builder /postmyrustache/target/release/postmyrustache /postmyrustache

# Set the entrypoint
ENTRYPOINT ["/postmyrustache"]

# Expose the necessary port
EXPOSE 3306