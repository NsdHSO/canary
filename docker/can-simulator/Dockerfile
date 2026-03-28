# Stage 1: Build the Rust ECU simulator
FROM rust:latest AS builder

WORKDIR /build

# Copy the Cargo project files
COPY Cargo.toml Cargo.lock* ./
COPY src ./src

# Build the release binary
RUN cargo build --release

# Stage 2: Runtime image
FROM alpine:latest

# Install CAN utilities and kernel modules
RUN apk add --no-cache \
    can-utils \
    iproute2 \
    kmod

# Create app directory
RUN mkdir -p /app

# Copy the compiled binary from builder stage
COPY --from=builder /build/target/release/ecu-simulator /app/ecu-simulator

# Copy setup scripts
COPY setup-vcan.sh /app/setup-vcan.sh
COPY run-simulator.sh /app/run-simulator.sh

# Make scripts and binary executable
RUN chmod +x /app/setup-vcan.sh /app/run-simulator.sh /app/ecu-simulator

# Default command
CMD ["/app/run-simulator.sh"]
