# What is this?

升级 is a popular Chinese trick-taking playing card game. Rules are available
[here](https://www.pagat.com/kt5/tractor.html). Due to the COVID-19
shelter-in-place, I've been unable to play the this game in person... so
I figured an online version would be worthwhile.

# Usage:

```
cargo run
```

The server is a self-contained static binary and does not terminate TLS. It
listens on 127.0.0.1:3030, and should only be exposed to an external network
behind a proxy that supports both HTTP and WebSocket protocols (only tested
with `nginx`).

# Development

```
cd frontend && yarn build
cd backend && cargo run --features dynamic
```

## Prettier
To format frontend code:

```
# Dry-run/check
yarn prettier --check

# Fix files, will overwrite files
yarn prettier --fix
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
- No implementation of throw penalties
- No mobile support
- Incomplete validity checking for forced-plays
- No player limit per game
- No overall player limit

# Demo

[https://robertying.com/shengji/](https://robertying.com/shengji/)
