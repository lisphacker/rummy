# AGENTS.md

## Project overview

This repository implements an online, turn-based **Rummy** game for **2–8 players**.

The application is written in Rust (2024 Edition) and uses:

- **Dioxus Fullstack** for the web client and full-stack integration;
- **Axum** and **Tokio** for the authoritative game server;
- typed **WebSockets** for active game sessions;
- **SQLx** with PostgreSQL for durable data;
- shared `serde`-serializable protocol types;
- a framework-independent `game-core` crate containing deterministic rules.

The browser is an untrusted presentation layer. The server owns the deck, hidden hands, turn order, legal-move validation, timers, scoring, room state, and reconnection state.

Read these documents before making non-trivial changes:

- `docs/code_setup.md`
- `docs/design.md`

## Primary engineering principles

1. **Server authoritative**
   - Clients send player intentions such as `DrawFromStock`, `CreateMeld`, and `Discard`.
   - Clients must never send replacement game state.
   - Never trust client-provided card ownership, turn status, score, timestamps, or room membership.

2. **Keep rules pure and deterministic**
   - `game-core` must not depend on Dioxus, Axum, Tokio, SQLx, browser APIs, wall-clock time, or network types.
   - Rules operate on explicit state, commands, and supplied randomness/results.
   - Prefer an interface shaped like:

     ```rust
     pub fn apply_command(
         state: &GameState,
         actor: PlayerId,
         command: GameCommand,
     ) -> Result<Transition, RuleError>;
     ```

   - `Transition` should contain the next state and emitted domain events, or events sufficient to derive the next state.

3. **Never leak hidden information**
   - The server's `GameState` and a player's `PlayerView` are different types.
   - A player receives only their own hand plus public information.
   - Spectators receive a separate `SpectatorView`.
   - Do not rely on CSS, Dioxus conditionals, or client code to conceal secret cards.

4. **Model Rummy variants explicitly**
   - Rummy is a family of games. Avoid scattering variant-specific `if` statements throughout the engine.
   - Put configurable rules in `RulesConfig` and isolate genuinely different behaviour behind small strategy functions or traits.
   - The first supported profile is `BasicRummy`.
   - New profiles must document their deviations and add conformance tests.

5. **Room ownership is single-threaded**
   - Each active room is owned by one Tokio task/actor.
   - Send commands to the room through channels.
   - Avoid a global `Arc<Mutex<HashMap<RoomId, GameState>>>` as the main mutation model.

6. **Reconnection is a core feature**
   - A WebSocket connection is not a player identity.
   - Authentication and seat ownership survive disconnects.
   - Every accepted room event has a monotonically increasing sequence number.
   - On reconnect, send either missing events or a fresh player-specific snapshot.

7. **Make illegal states difficult to represent**
   - Use newtypes for IDs.
   - Use enums for phases, draw sources, and turn stages.
   - Do not represent cards with ambiguous strings inside domain code.
   - Validate commands at the domain boundary.

## Workspace structure

Expected structure:

```text
.
├── AGENTS.md
├── Cargo.toml
├── Dioxus.toml
├── docs/
│   ├── code_setup.md
│   └── design.md
├── crates/
│   ├── game-core/
│   ├── protocol/
│   ├── game-server/
│   ├── game-ui/
│   └── test-support/
├── migrations/
├── assets/
└── tests/
```

### `crates/game-core`

Owns:

- cards, suits, ranks, decks and card identities;
- players, seats and turn order;
- rules configuration;
- stock and discard pile behaviour;
- meld validation;
- lay-off validation;
- going-out rules;
- scoring;
- deterministic commands, transitions and domain events;
- construction of public/player views from authoritative state, where practical without transport dependencies.

Must not own:

- authentication;
- database records;
- WebSocket sessions;
- HTTP endpoints;
- Dioxus components;
- Tokio tasks;
- system time.

### `crates/protocol`

Owns versioned client/server transport messages and DTOs shared by UI and server.

Protocol types should wrap or translate domain types rather than force transport concerns into `game-core`.

### `crates/game-server`

Owns:

- Axum/Dioxus server startup;
- authentication and sessions;
- lobby and room registry;
- room actors;
- WebSocket lifecycle;
- command routing;
- persistence and recovery;
- timers and disconnect policy;
- rate limiting, observability and moderation hooks.

### `crates/game-ui`

Owns Dioxus components, client routing, styling, accessibility, optimistic presentation, and client-side connection state.

The UI may predict animations but must reconcile with server messages. It must not decide whether a move is legal.

### `crates/test-support`

Owns reusable builders, deterministic test decks, seeded RNG helpers, fixtures and protocol test harnesses.

## Coding conventions

- Use stable Rust.
- Format with `cargo fmt --all`.
- Lint with `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- Prefer `thiserror` for library/domain errors and `anyhow` only at application boundaries.
- Avoid `unwrap`, `expect`, and indexing in production paths unless an invariant is locally proved and documented.
- Do not silently discard `Result`s.
- Keep public APIs documented.
- Prefer exhaustive `match` statements for domain enums.
- Use `tracing` rather than `println!` in server code.
- Do not log cards in hidden hands, session tokens, passwords, email addresses, or full WebSocket payloads.
- Store timestamps in UTC. Inject time into testable logic.
- Use `#[serde(tag = "type", content = "data")]` or another explicit stable representation for protocol enums.
- Add protocol versioning before public release.

## Domain rules and invariants

Unless overridden by a selected rules profile:

- A meld is a **set** of at least three cards of the same rank or a **run** of at least three consecutive cards of the same suit.
- A turn normally consists of drawing, optionally melding/laying off, and discarding.
- Only the active player may issue game actions.
- The same physical card cannot appear in more than one location.
- The union of hands, stock, discard pile, table melds and removed cards must equal the generated deck multiset.
- A command must be atomic: either the complete command succeeds or state remains unchanged.
- A player-specific view must not reveal another player's card identities.

Because rules vary, do not assume the following without reading `RulesConfig`:

- number of decks;
- number of jokers;
- hand size;
- whether ace is low, high, or configurable;
- whether the entire discard pile can be taken;
- whether taking a buried discard forces use of that card;
- whether laying off is allowed immediately or only after an initial meld;
- whether a final discard is required to go out;
- scoring values and match target.

## Commands and events

Prefer intention-oriented commands:

```rust
pub enum GameCommand {
    Draw { source: DrawSource },
    DrawFromDiscardDepth { depth: usize },
    CreateMeld { cards: Vec<CardId> },
    LayOff { meld_id: MeldId, cards: Vec<CardId> },
    Discard { card: CardId },
    GoOut,
}
```

Avoid commands such as `SetHand`, `SetTurn`, `UpdateGame`, or `SubmitState`.

Events should describe accepted facts:

```rust
pub enum GameEvent {
    CardDrawn { player: PlayerId, source: PublicDrawSource },
    MeldCreated { player: PlayerId, meld: Meld },
    CardsLaidOff { player: PlayerId, meld_id: MeldId, cards: Vec<Card> },
    CardDiscarded { player: PlayerId, card: Card },
    TurnAdvanced { player: PlayerId },
    PlayerWentOut { player: PlayerId },
    RoundScored { result: RoundResult },
}
```

Private consequences, such as the identity of a stock card drawn, must be filtered or represented differently per recipient.

## Testing requirements

For every rule change, add tests at the lowest suitable layer.

### Unit tests

Cover:

- set and run validation;
- duplicate physical-card rejection;
- ace boundary behaviour;
- jokers/wildcards when enabled;
- laying off on sets and both ends of runs;
- draw/discard turn-stage enforcement;
- legal and illegal going out;
- stock exhaustion/recycling;
- scoring;
- player-view secrecy.

### Property tests

Use `proptest` where useful. Important properties include:

- card conservation after every accepted command;
- no card appears in two zones;
- invalid commands do not mutate state;
- event sequence numbers strictly increase;
- a `PlayerView` never contains another player's hidden cards;
- shuffled decks contain exactly the configured card multiset.

### Integration tests

Cover:

- create room → join 2–8 players → start → play a round;
- reconnect and receive a correct snapshot;
- concurrent duplicate commands are accepted at most once;
- stale turn/sequence commands are rejected;
- server restart and room recovery, once persistence is implemented;
- WebSocket authorization and seat ownership.

### Determinism

Tests must use fixed decks or seeded RNG. Never make tests depend on production randomness or wall-clock sleeps.

## UI requirements

- Support mouse, touch and keyboard.
- Cards must be operable without drag-and-drop; clicking/tapping and keyboard actions are required.
- Do not encode suit using colour alone.
- Provide clear focus styles and accessible labels such as “Seven of Hearts”.
- Respect reduced-motion preferences.
- Show connection state and reconnection progress.
- Disable unavailable controls for clarity, but remember server validation remains mandatory.
- Avoid exposing hidden card identities in DOM attributes, debug text, serialized state or analytics.

## Database and migrations

- All schema changes require checked-in migrations.
- Prefer append-only game events plus periodic snapshots for resumable rooms.
- Persist idempotency keys or command IDs where needed to prevent duplicate application after retries.
- Do not put active mutable room state directly behind many unrelated database updates during a turn.
- Tests that touch PostgreSQL should run against an isolated database/schema.

## Security checklist

For networking or authentication changes, verify:

- authorization is checked server-side for every room command;
- room codes are not treated as authentication;
- command size and frequency are bounded;
- chat and player names are escaped/sanitized on output;
- cookies use appropriate `HttpOnly`, `Secure`, and `SameSite` settings;
- no hidden game data appears in logs or error messages;
- reconnect tokens are scoped and revocable;
- RNG used for shuffling is server-side and suitable for unpredictable shuffles;
- client-supplied sequence numbers cannot roll state backward.

## Workflow for Codex

Before editing:

1. Read the relevant design section.
2. Inspect nearby code and tests.
3. Identify whether the change belongs to domain, protocol, server, or UI.
4. State any rules-profile assumptions in the implementation or test name.

While editing:

1. Keep changes narrowly scoped.
2. Preserve crate boundaries.
3. Add or update tests with behaviour changes.
4. Avoid introducing a new dependency without clear value.
5. Do not reformat unrelated files.

Before finishing:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

Also run any database, browser or end-to-end tests relevant to the change.

In the final response, summarize:

- changed behaviour;
- affected crates/files;
- tests run and their results;
- unresolved risks or follow-up work.

## What not to do

- Do not move game legality into Dioxus components.
- Do not serialize authoritative `GameState` directly to clients.
- Do not add a global mutex as a shortcut around room ownership.
- Do not rely on WebSocket ordering alone for idempotency or reconnection.
- Do not implement a named Rummy variation from memory; document its rules and tests first.
- Do not make hidden cards available to spectators or observers.
- Do not couple core rules to one fixed player count, hand size, deck count or scoring table.
