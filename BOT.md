# Computer players

`shengji-bot` connects computer players to a room on a shengji server. They join like
ordinary players, see only what a human in their seat would see, and play Tractor or
Finding Friends under whatever rules the room is set to.

## Playing against bots

1. Start the server and open it in a browser (http://localhost:3030).
2. Enter your name. The address bar now ends in `#` followed by a 16-character room code.
3. Start the bots with that code (or the whole link):

   ```
   target\release\shengji-bot.exe --room <code> --names Ann,Bo,Cy,Di
   ```

4. In the browser, choose the game mode, number of decks and other settings, then start
   the game. You draw your own cards; the bots draw theirs when it's their turn.

Useful options:

| Option | Default | Meaning |
|---|---|---|
| `--names A,B,C` | `Bot-Ann` | One bot per name, seated in this order |
| `--think-ms N` | 1500 | Time budget per card play; more time means more simulated deals |
| `--samples N` | 64 | Maximum simulated deals per card play |
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

The bot doesn't attempt throws (multi-unit leads) yet, and its bidding, burying and
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
