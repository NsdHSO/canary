FROM rust:1.92 as builder

WORKDIR /app

# Copy manifests and source
COPY Cargo.toml Cargo.lock ./
COPY .cargo .cargo
COPY crates ./crates
COPY migration ./migration

# Build application (workspace requires -p flag)
RUN cargo build --release -p canary-api
RUN cargo build --release -p migration

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/canary-api /app/canary-api
COPY --from=builder /app/target/release/run_canary_migrations /app/migrate

# Create startup script that runs migrations then starts API
RUN echo '#!/bin/sh' > /app/start.sh && \
    echo '/app/migrate && exec /app/canary-api' >> /app/start.sh && \
    chmod +x /app/start.sh

EXPOSE ${PORT:-8080}

CMD ["/app/start.sh"]
