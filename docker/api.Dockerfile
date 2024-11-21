# Multi-stage build for Rust project using Alpine Linux
FROM rust:alpine3.20 AS builder

# Arguments for platform and username
ARG TARGETPLATFORM
ARG BUILDPLATFORM
ARG GIT_HTTPS_USERNAME
ARG GIT_HTTPS_PASSWORD

# Install build dependencies
RUN apk add --no-cache musl-dev openssl-dev protobuf-dev build-base pkgconfig \
    && apk add --no-cache openssl-libs-static git

# Set up Git credentials for HTTPS cloning without leaving them in a layer
RUN git config --global credential.helper store && \
    echo "https://${GIT_HTTPS_USERNAME}:${GIT_HTTPS_PASSWORD}@github.com" > /root/.git-credentials

# Set the working directory
WORKDIR /src

# Create a target directory to avoid permission issues
RUN mkdir -p ./target

# Copy project files
COPY src /src/
COPY bins /bins/
COPY crates /crates/
COPY Cargo.toml /Cargo.toml

# Build the Rust binary in release mode
RUN cargo build --release --package api --bin api

# Runner stage with a minimal base image
FROM alpine:3.20.2 AS runner

LABEL org.opencontainers.image.description="Service for TradeCrit"
LABEL org.opencontainers.image.licenses="GPL-3.0"

# Copy the binary from the builder stage
COPY --from=builder /target/release/api /usr/local/bin/

# Create a new user for running the service
RUN addgroup -S appuser && adduser -S appuser -G appuser

# Set ownership and permissions
RUN chown appuser:appuser /usr/local/bin/api

# Switch to the non-root user
USER appuser

# Expose the application's port
EXPOSE 50051

# Set the entrypoint and command
ENTRYPOINT ["api"]
CMD ["api"]
