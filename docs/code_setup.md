# Code setup

## 1. Purpose

This document describes how to create the initial Rust workspace for an online **Rummy** game supporting **2–8 players**.

The initial delivery target is a responsive browser application. The architecture keeps open the option of a Dioxus desktop or mobile client later, without coupling the game rules to any UI framework.

The recommended stack is:

- Rust stable;
- Dioxus 0.7 Fullstack;
- Axum and Tokio;
- typed WebSockets;
- SQLx and PostgreSQL;
- `serde` for protocol serialization;
- `tracing` for observability;
- `proptest` for rules invariants.

Dioxus Fullstack builds distinct client and server targets and uses Cargo features such as `web` and `server`. The Dioxus CLI can build both sides during development. See the official references at the end of this document.

## 2. Prerequisites

Install Rust through `rustup`, then add the WebAssembly target:

```bash
rustup toolchain install stable
rustup default stable
rustup target add wasm32-unknown-unknown
```

Install useful Cargo tooling:

```bash
cargo install cargo-binstall
cargo binstall dioxus-cli
cargo binstall cargo-nextest
cargo binstall sqlx-cli --no-default-features --features rustls,postgres
```

Verify the Dioxus environment:

```bash
dx doctor
dx --version
rustc --version
cargo --version
```

Required local services:

- PostgreSQL 16 or newer is recommended for development;
- Docker or Podman is optional but convenient.

Example development database:

```bash
docker run --name rummy-postgres \
  -e POSTGRES_USER=rummy \
  -e POSTGRES_PASSWORD=rummy \
  -e POSTGRES_DB=rummy \
  -p 5432:5432 \
  -d postgres:16
```

Create `.env` locally:

```dotenv
DATABASE_URL=postgres://rummy:rummy@localhost:5432/rummy
RUST_LOG=game_server=debug,tower_http=info
SESSION_SECRET=replace-with-at-least-32-random-bytes
```

Do not commit `.env` or production secrets.

## 3. Create the repository

```bash
mkdir rummy-game
cd rummy-game
git init
mkdir -p crates docs migrations assets tests
```

Create the workspace crates:

```bash
cargo new crates/game-core --lib
cargo new crates/protocol --lib
cargo new crates/game-server --bin
cargo new crates/game-ui --lib
cargo new crates/test-support --lib
```

The resulting structure should be:

```text
rummy-game/
├── AGENTS.md
├── Cargo.toml
├── Cargo.lock
├── Dioxus.toml
├── .env.example
├── .gitignore
├── assets/
├── crates/
│   ├── game-core/
│   ├── protocol/
│   ├── game-server/
│   ├── game-ui/
│   └── test-support/
├── docs/
│   ├── code_setup.md
│   └── design.md
├── migrations/
└── tests/
```

Commit `Cargo.lock` because this is an application workspace.

## 4. Root workspace manifest

Start with this root `Cargo.toml`:

```toml
[workspace]
resolver = "2"
members = [
    "crates/game-core",
    "crates/protocol",
    "crates/game-server",
    "crates/game-ui",
    "crates/test-support",
]
default-members = ["crates/game-server"]

[workspace.package]
edition = "2024"
license = "MIT OR Apache-2.0"
rust-version = "1.88"

[workspace.dependencies]
anyhow = "1"
async-trait = "0.1"
axum = { version = "0.8", features = ["ws", "macros"] }
chrono = { version = "0.4", default-features = false, features = ["clock", "serde"] }
dioxus = { version = "0.7", features = ["fullstack", "router"] }
futures-util = "0.3"
indexmap = { version = "2", features = ["serde"] }
proptest = "1"
rand = "0.9"
rand_chacha = "0.9"
secrecy = "0.10"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
smallvec = { version = "1", features = ["serde"] }
sqlx = { version = "0.8", default-features = false, features = [
    "runtime-tokio-rustls",
    "postgres",
    "migrate",
    "macros",
    "uuid",
    "chrono",
    "json",
] }
thiserror = "2"
tokio = { version = "1", features = ["macros", "rt-multi-thread", "signal", "sync", "time"] }
tower-http = { version = "0.6", features = ["compression-full", "cors", "request-id", "trace"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt", "json"] }
uuid = { version = "1", features = ["v4", "serde"] }

[workspace.lints.rust]
unsafe_code = "forbid"
missing_debug_implementations = "warn"

[workspace.lints.clippy]
all = "warn"
pedantic = "warn"
unwrap_used = "deny"
expect_used = "deny"
```

Version numbers above are a starting point, not a permanent compatibility promise. Let `cargo` resolve compatible releases and commit the resulting lock file. Review major/minor upgrades deliberately.

## 5. Crate manifests

### 5.1 `game-core`

`crates/game-core/Cargo.toml`:

```toml
[package]
name = "game-core"
version = "0.1.0"
edition.workspace = true
license.workspace = true
rust-version.workspace = true

[dependencies]
indexmap.workspace = true
rand.workspace = true
serde.workspace = true
smallvec.workspace = true
thiserror.workspace = true
uuid.workspace = true

[dev-dependencies]
proptest.workspace = true
rand_chacha.workspace = true
test-support = { path = "../test-support" }

[lints]
workspace = true
```

Keep this crate synchronous and framework-independent.

Suggested modules:

```text
src/
├── lib.rs
├── card.rs
├── command.rs
├── deck.rs
├── event.rs
├── id.rs
├── meld.rs
├── player.rs
├── rules/
│   ├── mod.rs
│   ├── basic.rs
│   ├── config.rs
│   ├── scoring.rs
│   └── validation.rs
├── state.rs
├── transition.rs
└── view.rs
```

### 5.2 `protocol`

`crates/protocol/Cargo.toml`:

```toml
[package]
name = "protocol"
version = "0.1.0"
edition.workspace = true
license.workspace = true
rust-version.workspace = true

[dependencies]
game-core = { path = "../game-core" }
serde.workspace = true
thiserror.workspace = true
uuid.workspace = true

[lints]
workspace = true
```

Suggested modules:

```text
src/
├── lib.rs
├── client.rs
├── server.rs
├── snapshot.rs
└── version.rs
```

Begin with an explicit protocol envelope:

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClientEnvelope {
    pub protocol_version: u16,
    pub command_id: uuid::Uuid,
    pub room_sequence: Option<u64>,
    pub message: ClientMessage,
}
```

### 5.3 `game-ui`

`crates/game-ui/Cargo.toml`:

```toml
[package]
name = "game-ui"
version = "0.1.0"
edition.workspace = true
license.workspace = true
rust-version.workspace = true

[dependencies]
dioxus.workspace = true
game-core = { path = "../game-core" }
protocol = { path = "../protocol" }
serde.workspace = true
thiserror.workspace = true
uuid.workspace = true

[features]
default = []
web = ["dioxus/web"]
desktop = ["dioxus/desktop"]
mobile = ["dioxus/mobile"]

[lints]
workspace = true
```

Suggested modules:

```text
src/
├── lib.rs
├── app.rs
├── components/
│   ├── card.rs
│   ├── connection_banner.rs
│   ├── discard_pile.rs
│   ├── hand.rs
│   ├── meld.rs
│   ├── player_seat.rs
│   └── stock.rs
├── pages/
│   ├── home.rs
│   ├── lobby.rs
│   ├── room.rs
│   └── rules.rs
├── state/
│   ├── connection.rs
│   └── room.rs
└── accessibility.rs
```

### 5.4 `game-server`

`crates/game-server/Cargo.toml`:

```toml
[package]
name = "game-server"
version = "0.1.0"
edition.workspace = true
license.workspace = true
rust-version.workspace = true

[dependencies]
anyhow.workspace = true
axum.workspace = true
chrono.workspace = true
dioxus.workspace = true
futures-util.workspace = true
game-core = { path = "../game-core" }
game-ui = { path = "../game-ui" }
protocol = { path = "../protocol" }
rand.workspace = true
secrecy.workspace = true
serde.workspace = true
serde_json.workspace = true
sqlx.workspace = true
thiserror.workspace = true
tokio.workspace = true
tower-http.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
uuid.workspace = true

[features]
default = []
web = ["dioxus/web", "game-ui/web"]
server = ["dioxus/server"]
desktop = ["dioxus/desktop", "game-ui/desktop"]
mobile = ["dioxus/mobile", "game-ui/mobile"]

[lints]
workspace = true
```

Suggested modules:

```text
src/
├── main.rs
├── app_state.rs
├── auth/
├── config.rs
├── db/
├── error.rs
├── lobby/
├── room/
│   ├── actor.rs
│   ├── command.rs
│   ├── handle.rs
│   ├── registry.rs
│   ├── recovery.rs
│   └── timers.rs
├── routes/
├── telemetry.rs
└── websocket/
    ├── connection.rs
    ├── handler.rs
    └── recipient_filter.rs
```

### 5.5 `test-support`

`crates/test-support/Cargo.toml`:

```toml
[package]
name = "test-support"
version = "0.1.0"
edition.workspace = true
license.workspace = true
rust-version.workspace = true
publish = false

[dependencies]
rand.workspace = true
rand_chacha.workspace = true
serde.workspace = true
uuid.workspace = true

[lints]
workspace = true
```

To avoid a dependency cycle, keep helpers that require `game-core` either in `game-core`'s test modules or create a one-way `game-fixtures` crate after the core API stabilizes. Initially, `test-support` should contain generic deterministic RNG and ID helpers only.

## 6. Dioxus configuration

Create `Dioxus.toml`:

```toml
[application]
name = "Rummy"
out_dir = "dist"
asset_dir = "assets"
sub_package = "game-server"

[web.app]
title = "Rummy"

[web.watcher]
watch_path = ["crates/game-ui", "crates/game-core", "crates/protocol", "assets"]

[web.resource]
style = ["/styles/main.css"]

[web.resource.dev]
style = []
script = []
```

The exact supported keys can change with the CLI, so validate with the installed Dioxus version and `dx --help`.

Create a minimal `assets/styles/main.css` with design tokens rather than component-specific styling:

```css
:root {
  color-scheme: light dark;
  --table: #1f6a49;
  --surface: #f8f6f1;
  --ink: #181818;
  --accent: #d19a35;
  --danger: #b42318;
  --card-ratio: 0.7142857;
}

* { box-sizing: border-box; }
body { margin: 0; font-family: system-ui, sans-serif; }
button, input, select { font: inherit; }
```

## 7. Initial domain types

Create strongly typed IDs in `game-core/src/id.rs`:

```rust
macro_rules! uuid_id {
    ($name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
            serde::Serialize, serde::Deserialize,
        )]
        pub struct $name(pub uuid::Uuid);

        impl $name {
            #[must_use]
            pub fn new() -> Self {
                Self(uuid::Uuid::new_v4())
            }
        }
    };
}

uuid_id!(GameId);
uuid_id!(RoundId);
uuid_id!(PlayerId);
uuid_id!(MeldId);
uuid_id!(CardId);
```

Represent a card's face explicitly. A standard card always has both a rank and a
suit, while a joker has neither:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Rank {
    Ace,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum CardFace {
    Standard { rank: Rank, suit: Suit },
    Joker,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Card {
    pub id: CardId,
    pub face: CardFace,
    pub deck_index: u8,
}
```

`CardFace` makes invalid combinations, such as a joker with a rank or suit,
unrepresentable. `Card` represents the physical card separately from its face;
therefore, a pair of identical-looking cards from two decks must still have
different `CardId`s.

## 8. Rules configuration

Rummy rules vary materially. Encode the profile rather than hard-code assumptions:

```rust
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RulesConfig {
    pub profile: RulesProfile,
    pub min_players: u8,
    pub max_players: u8,
    pub deck_count: u8,
    pub jokers_per_deck: u8,
    pub cards_per_player: u8,
    pub minimum_meld_size: u8,
    pub ace_policy: AcePolicy,
    pub discard_pickup: DiscardPickupRule,
    pub declaration: DeclarationRule,
    pub stock_recycle: StockRecycleRule,
    pub scoring: ScoringRules,
}
```

Provide a constructor such as:

```rust
impl RulesConfig {
    pub fn basic_rummy(player_count: u8) -> Result<Self, ConfigError> {
        // Choose a documented deck count and hand size for the player count.
        // Return an error rather than silently accepting unsupported counts.
    }
}
```

The project supports 2–8 seats. The canonical deal, declaration and scoring
behavior for the initial profile is defined in
[`docs/rules/BasicRummyV1.md`](rules/BasicRummyV1.md) and must be tested. Avoid
claiming one universal Rummy rule.

## 9. Command-processing skeleton

Create a single domain entry point:

```rust
pub fn apply_command(
    state: &GameState,
    actor: PlayerId,
    command: GameCommand,
) -> Result<Transition, RuleError> {
    validate_actor(state, actor)?;
    validate_phase(state, &command)?;

    match command {
        GameCommand::Draw { source } => draw(state, actor, source),
        GameCommand::Discard { card } => discard(state, actor, card),
        GameCommand::DeclareComplete { discard, melds } => {
            declare_complete(state, actor, discard, melds)
        }
        GameCommand::SubmitForScoring { melds, unmatched } => {
            submit_for_scoring(state, actor, melds, unmatched)
        }
    }
}
```

Do not mutate shared state inside validation. Construct a transition only after the complete command is valid.

## 10. Room actor skeleton

The room actor is the sole owner of active room state:

```rust
pub struct RoomHandle {
    tx: tokio::sync::mpsc::Sender<RoomMessage>,
}

pub enum RoomMessage {
    PlayerCommand {
        player_id: PlayerId,
        command_id: uuid::Uuid,
        expected_sequence: Option<u64>,
        command: GameCommand,
        reply: tokio::sync::oneshot::Sender<Result<CommandReceipt, RoomError>>,
    },
    Connected { player_id: PlayerId, connection: ConnectionHandle },
    Disconnected { player_id: PlayerId, connection_id: uuid::Uuid },
    TimerExpired { timer: RoomTimer },
    Shutdown,
}
```

Processing outline:

```rust
async fn run_room(mut room: ActiveRoom, mut rx: Receiver<RoomMessage>) {
    while let Some(message) = rx.recv().await {
        match message {
            RoomMessage::PlayerCommand { /* ... */ } => {
                // 1. Authorize seat.
                // 2. Check command id/idempotency.
                // 3. Check expected sequence if supplied.
                // 4. Apply game-core command.
                // 5. Persist event(s)/snapshot.
                // 6. Commit in-memory transition.
                // 7. Broadcast recipient-specific messages.
                // 8. Reply with receipt.
            }
            // ...
        }
    }
}
```

Do not hold a mutex guard across `.await`.

## 11. WebSocket protocol

Use one active room connection per client tab/app instance.

Suggested messages:

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ClientMessage {
    Resume {
        room_id: GameId,
        last_seen_sequence: Option<u64>,
    },
    GameCommand(GameCommand),
    Ping { nonce: u64 },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ServerMessage {
    Snapshot(PlayerSnapshot),
    Event(SequencedPlayerEvent),
    CommandAccepted { command_id: uuid::Uuid, sequence: u64 },
    CommandRejected { command_id: uuid::Uuid, error: PublicCommandError },
    ConnectionState(ConnectionState),
    Pong { nonce: u64 },
}
```

Filter every domain event for each recipient. A private stock draw might produce:

- to the drawing player: the exact card;
- to opponents: only that the player drew one card from the stock;
- to spectators: the same public event as opponents.

## 12. Database setup

Create an initial migration:

```bash
sqlx migrate add initial_schema
```

Recommended initial tables:

- `users`;
- `sessions`;
- `rooms`;
- `room_members`;
- `games`;
- `game_events`;
- `game_snapshots`;
- `processed_commands`.

A minimal event table shape:

```sql
CREATE TABLE game_events (
    game_id UUID NOT NULL,
    sequence BIGINT NOT NULL,
    event_type TEXT NOT NULL,
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (game_id, sequence)
);
```

Do not persist recipient-filtered events as the canonical event stream. Store authoritative domain events in protected server storage and generate player-specific views when serving clients.

After migrations:

```bash
sqlx database create
sqlx migrate run
cargo sqlx prepare --workspace
```

Commit `.sqlx/` if CI or production builds use SQLx offline mode.

## 13. Authentication development path

Use a staged approach:

1. anonymous guest identity stored in a signed, secure session cookie;
2. optional account upgrade later;
3. room invitation code determines discoverability, not authorization;
4. joining a seat creates a server-side membership record;
5. reconnect uses the authenticated identity to reclaim that seat.

Do not put player identity solely in local storage or accept a `player_id` supplied without authentication.

## 14. First vertical slice

Implement this sequence before animations or advanced variants:

1. Home page creates a room.
2. Creator receives a room code/link.
3. A second browser joins.
4. Both mark ready.
5. Server starts a two-player Basic Rummy round with a fixed test deck.
6. Active player draws from stock.
7. Active player discards.
8. Opponent sees public changes without learning the drawn card.
9. Refreshing either browser reconnects and restores the correct view.
10. Replace the fixed deck with secure server-side shuffling.

Then add:

- creating melds;
- laying off;
- complete-hand declaration and scoring submissions;
- 3–8 player layouts;
- timers and disconnect policy;
- bots for local testing;
- match history.

## 15. Developer commands

Add a `justfile` or `Makefile`. Example `justfile`:

```make
set dotenv-load := true

serve:
    dx serve --web

fmt:
    cargo fmt --all

check:
    cargo check --workspace --all-targets --all-features

lint:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
    cargo nextest run --workspace --all-features

test-doc:
    cargo test --workspace --doc

migrate:
    sqlx migrate run

prepare-sqlx:
    cargo sqlx prepare --workspace
```

## 16. CI baseline

CI should run:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

Also add:

- PostgreSQL service container and migration test;
- `cargo audit` or equivalent dependency review;
- WASM/web build;
- Dioxus production build;
- browser end-to-end tests once the first vertical slice exists.

## 17. Definition of setup complete

The setup phase is complete when:

- every crate builds;
- dependency direction follows the design;
- a server starts with structured logging;
- the Dioxus home page loads;
- a typed WebSocket connects;
- PostgreSQL migrations run;
- `game-core` can start a deterministic two-player round;
- one command passes through UI/protocol/server/core and returns a recipient-filtered event;
- refresh/reconnect restores the player's view;
- format, lint and tests pass in CI.

## References

- Dioxus 0.7 getting started: https://dioxuslabs.com/learn/0.7/getting_started/
- Dioxus Fullstack project setup: https://dioxuslabs.com/learn/0.7/essentials/fullstack/project_setup/
- Dioxus Fullstack WebSockets: https://dioxuslabs.com/learn/0.7/essentials/fullstack/websockets/
- Dioxus CLI configuration: https://dioxuslabs.com/learn/0.7/guides/tools/configure/
- General Rummy description and Basic Rummy rules: https://en.wikipedia.org/wiki/Rummy
