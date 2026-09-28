# Computer players

Computer players join a room like ordinary players, see only what a human in their seat
would see, and play Tractor or Finding Friends under whatever rules the room is set to.

## Playing against bots

1. Run `pwsh -File play-local.ps1`. It starts the server and opens http://localhost:3030.
   Add `-Build` after pulling changes, and `-Stop` to shut the server down.
2. Enter your name and join.
3. Under **Computer players**, pick a level and click **Fill to 4 players with bots** (or
   **Add a bot** for each seat). Kick a bot to remove it.
4. Choose the game mode, decks and other settings, then click **Start game**.

Levels:

| Level | How it plays |
|---|---|
| Beginner (learning) | Simple rules, with a deliberate mistake about a third of the time |
| Easy | Simple rules, no simulation |
| Intermediate | 16 simulated deals per play, about 0.5 s |
| Hard | 64 simulated deals, about 1.5 s |
| Expert | 200 simulated deals, about 4 s |

**Hint** appears on your turn. It simulates 200 deals from your point of view (it never
sees other hands) and shows the best play, the chance your side wins the hand with each
option, and the defenders' average points. **Select suggested cards** selects them for
you. While you're burying the kitty, Hint suggests what to bury and which friend to call,
and can move the cards for you.

## Playing offline

On the join screen, click **Practice offline against computer players** (or open the
site with `#offline-practice` at the end of the address). The engine and bots then run
inside the page, in a background worker compiled to WebAssembly, with no server needed. The
game is saved in the browser as you play and resumes after a reload. Offline is always a
single human against bots; the lobby, levels and Hint work the same way.

The pieces:

- `local-game/` runs a game with bots in-process. It accepts the same JSON messages a
  client sends the server and returns the same messages the server would send, and it can
  save and restore a game (`snapshot`/`restore`).
- `frontend/shengji-wasm` exposes it to JavaScript as `OfflineGame`.
- `frontend/src/offlineWorker.ts` hosts it in a Web Worker so the page stays responsive
  while bots think. `OfflineProvider.tsx` stands in for the websocket connection.

WebAssembly runs single-threaded, so bots simulate fewer deals in the same time than
server-side bots do. This is also the basis for the planned iOS app.

## Command-line bots

`shengji-bot` seats bots from the command line, for example to join a server elsewhere:

```
target\release\shengji-bot.exe --room <code> --names Ann,Bo,Cy --difficulty hard
```

Useful options:

| Option | Default | Meaning |
|---|---|---|
| `--names A,B,C` | `Bot-Ann` | One bot per name, seated in this order |
| `--difficulty L` | `hard` | beginner, easy, intermediate, hard or expert |
| `--think-ms N` | by level | Time budget per card play; more time means more simulated deals |
| `--samples N` | by level | Maximum simulated deals per card play |
| `--threads N` | cores, up to 8 | Simulation threads per bot |
| `--start-new-games` | off | Bots start the next hand themselves when one of them is landlord |
| `--no-finish-tricks` | off | Leave finishing completed tricks to the humans |
| `--url` | `ws://127.0.0.1:3030/api` | Server websocket address |
| `--quiet` | off | Don't print each decision |

## How it plays

The logic lives in `core/src/bot/`:

- `play.rs` handles card play with perfect-information Monte Carlo. For each decision it
  deals the unseen cards to the other players (and the kitty) at random, consistent with
  hand sizes and the suits each player has shown out of. It then plays every candidate
  move out to the end of the hand with a fast heuristic policy, scoring each by the level
  outcome plus the point margin, and picks the move with the best average. In Finding
  Friends the sampled deal also decides who holds the called card, so the hidden partner
  is handled naturally.
- `setup.rs` handles bidding (bid a level card once enough of the hand is drawn and the
  suit would be strong), burying the kitty (void short side suits, keep trump, aces,
  pairs and points), and calling friends (an ace, or a king if needed, in a short side
  suit, skipping past the copies the bot holds).
- `heuristics.rs` has shared card helpers.

When leading, it also considers throws of cards that card counting says nobody can beat,
and the simulations check whether each throw is safe. Its bidding, burying and
friend calls are rule-based rather than simulated.

## Testing without a server

`core/examples/bot_selfplay.rs` plays whole games between bots inside the engine, with
each bot seeing only its own redacted view:

```
cargo run --release -p shengji-core --example bot_selfplay -- 5 3 4 300
cargo run --release -p shengji-core --example bot_selfplay -- 4 2 30 200 tractor mixed
```

The arguments are players, decks, hands, milliseconds per move, then `tractor` (default is
Finding Friends) and `mixed`. In `mixed` mode, odd seats play by the heuristic alone, which
measures what the simulation adds.
