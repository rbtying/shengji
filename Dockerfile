ARG PLATFORM=$BUILDPLATFORM

FROM --platform=$PLATFORM ghcr.io/rbtying/yarn-wasm-rust-build-image:master as wasmbase

# Create a workspace recipe.json to pre-fetch and pre-compile dependencies
FROM wasmbase as planner
WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Pre-compile frontend wasm dependencies
FROM wasmbase as frontend-cacher
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json --target=wasm32-unknown-unknown -p shengji-wasm

# Download Yarn dependencies
FROM wasmbase as frontend-deps-fetch
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

# Create a workspace recipe.json to pre-fetch and pre-compile dependencies, but
# without shengji-wasm because this is the backend
FROM wasmbase as planner-no-wasm
WORKDIR /app
COPY . .
RUN rm -r frontend
RUN cat Cargo.toml | grep -v frontend > Cargo2.toml && mv Cargo2.toml Cargo.toml
RUN cargo chef prepare --recipe-path recipe.json

# Compile backend for amd64 and arm64, because TARGETPLATFORM can't be used in
# the build stages
FROM --platform=$PLATFORM messense/rust-musl-cross:x86_64-musl as amd64
ARG PLATFORM
RUN case "$PLATFORM" in \
  "linux/arm64") echo "aarch64-unknown-linux-gnu" > /host-target ;; \
  "linux/amd64") echo "x86_64-unknown-linux-gnu" > /host-target ;; \
  *) exit 1 ;; \
esac
ARG TARGETPLATFORM
RUN case "$TARGETPLATFORM" in \
  "linux/arm64") exit 0 ;; \
  "linux/amd64") cargo install cargo-chef --target $(cat /host-target) ;; \
  *) exit 1 ;; \
esac
WORKDIR /app
COPY --from=planner-no-wasm /app/recipe.json recipe.json
RUN case "$TARGETPLATFORM" in \
  "linux/arm64") mkdir target ;; \
  "linux/amd64") cargo chef cook --release --recipe-path recipe.json --target x86_64-unknown-linux-musl ;; \
  *) exit 1 ;; \
esac
COPY Cargo.toml .
COPY Cargo.lock .
COPY core ./core
COPY frontend ./frontend
COPY backend/ ./backend
COPY storage ./storage
COPY favicon ./favicon
COPY --from=frontend-builder /app/frontend/dist /app/frontend/dist/
RUN case "$TARGETPLATFORM" in \
  "linux/arm64") mkdir -p target/x86_64-unknown-linux-musl/release && touch target/x86_64-unknown-linux-musl/release/shengji ;; \
  "linux/amd64") cargo build --release --bin shengji --target x86_64-unknown-linux-musl ;; \
  *) exit 1 ;; \
esac

FROM --platform=$PLATFORM messense/rust-musl-cross:aarch64-musl as arm64
ARG PLATFORM
RUN case "$PLATFORM" in \
  "linux/arm64") echo "aarch64-unknown-linux-gnu" > /host-target ;; \
  "linux/amd64") echo "x86_64-unknown-linux-gnu" > /host-target ;; \
  *) exit 1 ;; \
esac
ARG TARGETPLATFORM
RUN case "$TARGETPLATFORM" in \
  "linux/amd64") exit 0 ;; \
  "linux/arm64") cargo install cargo-chef --target $(cat /host-target);; \
  *) exit 1 ;; \
esac
WORKDIR /app
COPY --from=planner-no-wasm /app/recipe.json recipe.json
RUN case "$TARGETPLATFORM" in \
  "linux/amd64") mkdir target ;; \
  "linux/arm64") cargo chef cook --release --recipe-path recipe.json --target aarch64-unknown-linux-musl ;; \
  *) exit 1 ;; \
esac
COPY Cargo.toml .
COPY Cargo.lock .
COPY core ./core
COPY frontend ./frontend
COPY backend/ ./backend
COPY storage ./storage
COPY favicon ./favicon
COPY --from=frontend-builder /app/frontend/dist /app/frontend/dist/
RUN case "$TARGETPLATFORM" in \
  "linux/arm64") cargo build --release --bin shengji --target aarch64-unknown-linux-musl ;; \
  "linux/amd64") mkdir -p target/aarch64-unknown-linux-musl/release && touch target/aarch64-unknown-linux-musl/release/shengji ;; \
  *) exit 1 ;; \
esac

# Merge them
FROM alpine as merged
ARG TARGETPLATFORM
COPY --from=amd64 /app/target/x86_64-unknown-linux-musl/release/shengji /shengji.x86_64
COPY --from=arm64 /app/target/aarch64-unknown-linux-musl/release/shengji /shengji.aarch64
RUN case "$TARGETPLATFORM" in \
  "linux/arm64") ln /shengji.aarch64 /shengji ;; \
  "linux/amd64") ln /shengji.x86_64 /shengji ;; \
  *) exit 1 ;; \
esac

# Executable
FROM alpine
COPY --from=merged /shengji /shengji

ENTRYPOINT ["/shengji"]
