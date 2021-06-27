FROM node:current-slim as base
RUN apt-get update && apt-get -y install curl build-essential
# Install Rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain stable -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup target add wasm32-unknown-unknown
# Install wasm-pack
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
RUN yarn global add rimraf webpack webpack-cli
# Install cargo-chef
RUN cargo install cargo-chef

# Create a workspace recipe.json to pre-fetch and pre-compile dependencies
FROM base as planner
WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Pre-compile frontend wasm dependencies
FROM base as frontend-cacher
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json --target=wasm32-unknown-unknown -p shengji-wasm

# Pre-compile backend dependencies
FROM base as backend-cacher
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Download Yarn dependencies
FROM base as frontend-deps-fetch
COPY frontend/package.json ./
COPY frontend/yarn.lock ./
RUN yarn install

# Actually build the frontend
FROM frontend-deps-fetch as frontend-builder
WORKDIR /app
COPY --from=frontend-cacher /app/target /app/target
# Run the actual build
COPY Cargo.toml .
COPY Cargo.lock .
COPY core ./core
COPY backend/backend-types ./backend/backend-types/
COPY frontend ./frontend
COPY backend/Cargo.toml ./backend/Cargo.toml
COPY backend/src/main.rs ./backend/src/
COPY storage ./storage
WORKDIR /app/frontend
RUN yarn build

# Actually build the backend
FROM base as builder
WORKDIR /app
COPY --from=backend-cacher /app/target target
COPY Cargo.toml .
COPY Cargo.lock .
COPY core ./core
COPY frontend ./frontend
COPY backend/ ./backend
COPY storage ./storage
COPY favicon ./favicon
COPY --from=frontend-builder /app/frontend/dist /app/frontend/dist/
WORKDIR /app
RUN cargo build --release --bin shengji

# Executable
FROM gcr.io/distroless/cc:debug
WORKDIR /app
COPY --from=builder /app/target/release/shengji .
ENTRYPOINT ["/app/shengji"]
