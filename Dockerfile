# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------
ARG APP_NAME="api-endpoint"
ARG TARGET="x86_64-unknown-linux-musl"
FROM --platform=linux/amd64 rust:1.77.0-alpine as builder

ARG APP_NAME
ARG TARGET
WORKDIR /usr/src/$APP_NAME

# Create blank project
RUN USER=root cargo new $APP_NAME
RUN apk update && apk upgrade
RUN apk add postgresql
RUN apk add alpine-sdk
RUN apk add --no-cache make protobuf-dev

## Install target platform (Cross-Compilation) --> Needed for Alpine
RUN rustup target add $TARGET

# Now copy in the rest of the sources
RUN mkdir -p /usr/src/common
COPY ./common ../common
COPY ./$APP_NAME/ .

# This is the actual application build.
RUN cargo build --target $TARGET --release

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------
FROM alpine:3.18.0 AS runtime 
ARG APP_NAME
ARG TARGET

# Copy application binary from builder image
COPY --from=builder /usr/src/$APP_NAME/target/$TARGET/release/$APP_NAME /usr/local/bin

# Run the application
CMD ["/usr/local/bin/api-endpoint"]
