# Use cargo-chef
FROM lukemathwalker/cargo-chef:latest-rust-1-alpine AS chef
# Set the working directory
WORKDIR /app

######################
# Prepare the recipe #
######################
FROM chef AS preparer
COPY Cargo.toml Cargo.lock ./
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

###################
# Cook the recipe #
###################
FROM chef AS cooker
# Copy the recipe
COPY --from=preparer /app/recipe.json recipe.json
# Build dependencies
RUN cargo chef cook --release --recipe-path recipe.json
# Copy the source code
COPY src/ ./src/
COPY Cargo.toml Cargo.lock ./
# Build the application
RUN cargo build --release

###################################
# Use alpine linux as the runtime #
###################################
FROM alpine:latest AS runtime
# Set the working directory
WORKDIR /app
# Copy the built binary
COPY --from=cooker /app/target/release/awb /usr/local/bin
# Copy translations
COPY lang/ ./lang/
# Set the entrypoint for the app
ENTRYPOINT ["/usr/local/bin/awb"]
