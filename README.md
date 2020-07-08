# What is this?

升级 is a popular Chinese trick-taking playing card game, also known as tractor, finding friends, fighting for 100 points, 80 points, etc. Rules are available
[here](https://robertying.com/shengji/rules). Due to the COVID-19
shelter-in-place, I've been unable to play the this game in person... so
I figured an online version would be worthwhile.

# Usage:

```
cd frontend && yarn build && cd .. && cd backend && cargo run
```

The server is a self-contained static binary and does not terminate TLS. It
listens on 127.0.0.1:3030, and should only be exposed to an external network
behind a proxy that supports both HTTP and WebSocket protocols (only tested
with `nginx`).

# Development

```
cd frontend && yarn watch
cd backend && cargo run --features dynamic
```

## Generating JSON
A mapping of card data is generated from the server. It's checked in at
`src/generated/cards.json`. To update it, start up the server and run

```
yarn download-cards-json
```

## Prettier
To format frontend code:

```
# Dry-run/check
yarn prettier --check

# Fix files, will overwrite files
yarn prettier --write
```

## Lint
To run tslint:

```
cd frontend && yarn lint
```

And clippy:
```
cargo clippy
```

## Tests
To run tests:

### Frontend:
```
cd frontend && yarn test
```

### Backend:
```
cargo test
```

# Technical details
The entire state of each game is stored in the memory of the server process.
Restarting the game kicks all players, and games are automatically closed when
all players have disconnected. The bulk of the game logic is implemented in the
server, but players are expected to keep each other in check -- the server does
not validate moves in their entirety.

For simplicity, the game is written in Rust and Javascript, linking in Warp as
the WebSocket/HTTP server implementation and using React from a CDN.

# Known issues
- No mobile support
- Incomplete validity checking for forced-plays
- No player limit per game
- No overall player limit

# Play online!

[https://robertying.com/shengji/](https://robertying.com/shengji/)
