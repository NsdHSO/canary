FROM alpine:latest

# Install CAN utilities and kernel modules
RUN apk add --no-cache \
    can-utils \
    iproute2 \
    kmod

# Create app directory
RUN mkdir -p /app

# Copy setup scripts
COPY setup-vcan.sh /app/setup-vcan.sh
COPY run-simulator.sh /app/run-simulator.sh

# Make scripts executable
RUN chmod +x /app/setup-vcan.sh /app/run-simulator.sh

# Default command
CMD ["/app/run-simulator.sh"]
