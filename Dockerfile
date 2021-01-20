
# Use cargo-chef to maximize layer re-use when rebuilding in Docker.

FROM rust:alpine as planner
RUN apk add --no-cache musl-dev
WORKDIR app
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# FROM rust:alpine as cacher
# RUN apk add --no-cache musl-dev
# WORKDIR app
# RUN cargo install cargo-chef
# COPY --from=planner /app/recipe.json recipe.json
# RUN cargo chef cook --release --recipe-path recipe.json

# Build the actual shengji binary

# FROM rust:alpine as builder
# RUN apk add --no-cache musl-dev
# WORKDIR app
# COPY . .
# COPY --from=frontend-builder /app/frontend/dist/ /app/frontend/dist
# COPY --from=cacher /app/target target
# COPY --from=cacher $CARGO_HOME $CARGO_HOME
# RUN cargo build --release --bin shengji

# # Construct the final image.
# FROM alpine as runtime
# WORKDIR app
# COPY --from=builder /app/target/release/shengji /usr/local/bin
# ENTRYPOINT ["/usr/local/bin/shengji"]

# Do the frontend build

FROM node:current-alpine as frontend-builder
RUN apk add --no-cache curl
RUN yarn global add rimraf webpack webpack-cli wasm-pack

WORKDIR app
COPY frontend/package.json ./
COPY frontend/yarn.lock ./
RUN yarn install
COPY frontend ./
COPY backend/backend-types ./backend/
COPY core ./
WORKDIR frontend
RUN yarn build
