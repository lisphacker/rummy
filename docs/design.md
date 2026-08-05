# Online Rummy design

## 1. Executive summary

This project is an online, turn-based **Rummy** game for **2–8 players**, initially delivered as a responsive web application.

“Rummy” refers to a family of games centred on forming melds. Common melds are:

- **sets**: at least three cards of the same rank;
- **runs**: at least three consecutive cards of the same suit.

A typical turn draws one or more cards according to the selected rules, optionally creates or extends melds, and ends with a discard. However, deck count, cards dealt, discard-pile pickup, jokers, ace handling, laying off, going out and scoring vary substantially among Rummy variants.

The product therefore has two layers:

1. a reusable, configurable **Rummy engine**;
2. an initial **Basic Rummy** rules profile used by the first playable release.

The server is authoritative. Clients never receive other players' hidden cards and cannot decide legal moves.

## 2. Goals

### 2.1 Product goals

- Allow 2–8 people to create or join a private room through a link or room code.
- Require no installation for the initial web release.
- Make play understandable on desktop, tablet and mobile.
- Recover gracefully from page refreshes and temporary network loss.
- Support a complete Basic Rummy round and multi-round match.
- Keep rule options explicit enough to add selected Rummy variants later.
- Make the game usable with mouse, touch and keyboard.

### 2.2 Engineering goals

- Pure, deterministic and heavily tested game rules.
- Strong separation between authoritative state and recipient-specific views.
- Typed client/server messages.
- Serialized room mutation through one actor/task.
- Event sequence numbers and idempotent command handling.
- Recoverable active games using snapshots and/or an event log.
- Horizontal scaling later without redesigning the core rules.

## 3. Non-goals for the first release

- Real-money gambling, wagering or cash prizes.
- Public ranked matchmaking.
- Tournament brackets.
- Voice or video chat.
- User-authored rule scripting.
- Every regional Rummy variant.
- Native desktop or mobile packaging.
- Spectator mode unless it falls out cheaply from player-view filtering.
- Anti-cheat against collusion between players outside the application.

## 4. Rules scope and assumptions

### 4.1 Rummy is configurable

The game engine must not treat one internet rules page as the universal definition of Rummy. A `RulesConfig` selected when creating a room determines all variable behaviour.

The initial profile is named `BasicRummyV1`. The profile is versioned so saved games remain interpretable after future rule changes.

### 4.2 Proposed `BasicRummyV1` profile

The following is a product definition for this application, informed by common Basic/Straight Rummy rules. It must be shown clearly to players rather than presented as the only possible ruleset.

#### Players and deck

- Minimum players: 2.
- Maximum players: 8.
- One standard 52-card deck for 2–3 players.
- Two standard 52-card decks for 4–8 players.
- Jokers disabled in the first release.
- Identical-looking cards from different decks remain distinct physical cards.

The cited Basic Rummy description commonly uses one pack for two or three players and two packs for larger groups up to seven. Supporting eight players with two packs is a deliberate product extension and must be tested for sufficient stock under the selected deal sizes.

#### Cards dealt

Proposed deal sizes:

| Players | Cards each | Decks |
|---:|---:|---:|
| 2 | 10 | 1 |
| 3 | 10 | 1 |
| 4 | 10 | 2 |
| 5 | 10 | 2 |
| 6 | 10 | 2 |
| 7 | 10 | 2 |
| 8 | 10 | 2 |

This intentionally provides a consistent ten-card experience. It differs from some Basic Rummy tables that reduce hand size when using one pack. The UI must label this room profile explicitly.

An alternative conservative profile may later reproduce a published deal table exactly.

#### Turn sequence

A turn has these stages:

1. `AwaitingDraw`
2. `AfterDraw`
3. zero or more meld/lay-off actions
4. `AwaitingDiscard`
5. discard and advance turn

A player may:

- draw the top card from the face-down stock; or
- draw from the discard pile according to the configured pickup rule;
- create any number of valid melds after drawing;
- lay off cards on existing table melds if permitted;
- discard one card to complete the turn.

#### Melds

- A set contains at least three cards of the same rank.
- A run contains at least three consecutive ranks of the same suit.
- Aces may be low (`A-2-3`) or high (`Q-K-A`) but may not wrap (`K-A-2`).
- A set may not contain two cards of the same suit when using multiple decks, unless a future profile explicitly permits duplicate-suit sets. For `BasicRummyV1`, duplicate suits in one set are rejected.
- A run may contain only one physical copy of a given rank/suit position.
- Table melds remain public.

The duplicate-suit rule is a product choice needed when two decks are present; it avoids sets such as two Kings of Hearts plus a King of Clubs.

#### Laying off

- Once a player has created at least one meld in the current or an earlier turn, they may add cards to any valid public meld.
- A set can be extended up to four suits under `BasicRummyV1`.
- A run can be extended at either end.
- A lay-off is atomic: all submitted cards must produce a valid resulting meld.

#### Discard pickup

For the first release:

- A player may take only the top card of the discard pile.
- The drawn discard becomes part of the player's hand.
- The player may not immediately discard the exact same physical card on that turn.

A future profile can support taking a buried discard plus all cards above it, possibly requiring immediate use of the selected buried card.

#### Going out

- A player goes out only by discarding their final card.
- A player cannot end the round solely by placing all remaining cards into melds or lay-offs.
- The final discard must itself be legal under the normal turn rules.

This removes an ambiguity found across Rummy variants and gives the UI a consistent final action.

#### Stock exhaustion

When the stock becomes empty:

- preserve the top card of the discard pile;
- move the remaining discard pile into a new stock;
- shuffle the new stock using secure server-side randomness;
- continue play;
- limit recycling to a configurable number to avoid pathological matches.

The first release uses a maximum of two stock recycles. If nobody goes out after that, end the round as blocked and score remaining hands according to the profile.

#### Scoring

When a player goes out:

- numbered cards score face value;
- J, Q and K score 10;
- A scores 1;
- no jokers exist in `BasicRummyV1`;
- each opponent's unmelded hand value is awarded to the winner;
- cards already placed in public melds do not count against a player.

A match ends when a player reaches the configured target, initially 100 points. Ties after a blocked round do not end the match unless someone has reached the target; highest total wins, with a shared result if totals remain equal.

### 4.3 Rules still requiring playtesting

- Ten cards for all 4–8 player games may create long turns or deplete the stock quickly.
- Whether laying off should be allowed in the same turn as a player's first meld.
- Whether players should be allowed to rearrange existing table melds.
- Whether a blocked round should award lowest-hand points rather than no winner.
- Whether two stock recycles are too many for larger rooms.
- Turn timer defaults and disconnected-player policy.

Keep these values configurable and instrument games before finalising them.

## 5. System context

```text
┌──────────────────────┐
│ Dioxus web client    │
│                      │
│ Lobby and game UI    │
│ Local presentation   │
│ WebSocket client     │
└──────────┬───────────┘
           │ HTTPS / WSS
           ▼
┌──────────────────────────────────────┐
│ Dioxus Fullstack / Axum server       │
│                                      │
│ Auth and sessions                    │
│ Lobby and room registry              │
│ Room actors                          │
│ Recipient filtering                  │
│ Persistence and recovery             │
└───────────────┬──────────────────────┘
                │ SQL
                ▼
┌──────────────────────────────────────┐
│ PostgreSQL                           │
│ Users, rooms, membership, events,    │
│ snapshots, scores and command IDs    │
└──────────────────────────────────────┘
```

## 6. Component design

### 6.1 `game-core`

The domain crate defines:

- cards and physical card identities;
- deck generation;
- rules profiles/configuration;
- game and round state;
- commands;
- validation and transitions;
- meld operations;
- scoring;
- public and player-specific projections where transport-neutral.

It is deterministic except where the caller supplies shuffled card order or an RNG abstraction.

### 6.2 `protocol`

The protocol crate defines stable message envelopes shared by server and clients.

It is responsible for:

- protocol version;
- command IDs;
- expected room sequence;
- reconnect/resume requests;
- player snapshots;
- public errors;
- recipient-filtered events.

Protocol DTOs may intentionally differ from domain structures to avoid leaking implementation details and to support version migration.

### 6.3 `game-server`

The server owns:

- identity and seat authorization;
- room creation and joining;
- room actor lifecycle;
- secure shuffle generation;
- command idempotency;
- timers;
- event persistence;
- snapshots and recovery;
- WebSocket fan-out;
- recipient filtering;
- metrics and moderation hooks.

### 6.4 `game-ui`

The UI owns:

- route and page structure;
- lobby controls;
- responsive table layout;
- card selection and ordering;
- animation and sound preferences;
- connection/reconnection indicators;
- accessible alternatives to dragging;
- local view-model state.

The UI may show a proposed meld before submitting it, but the server decides whether it is valid.

## 7. Authoritative state model

A simplified authoritative model:

```rust
pub struct GameState {
    pub game_id: GameId,
    pub rules: RulesConfig,
    pub phase: GamePhase,
    pub round: RoundState,
    pub match_scores: BTreeMap<PlayerId, u32>,
    pub sequence: u64,
}

pub struct RoundState {
    pub round_id: RoundId,
    pub dealer: SeatIndex,
    pub active_seat: SeatIndex,
    pub turn_stage: TurnStage,
    pub players: Vec<PlayerRoundState>,
    pub stock: Vec<Card>,
    pub discard: Vec<Card>,
    pub melds: BTreeMap<MeldId, TableMeld>,
    pub stock_recycles: u8,
    pub pending_draw_restriction: Option<DrawRestriction>,
}
```

### 7.1 Card conservation invariant

For every active round, each configured physical card appears exactly once in one of:

- stock;
- discard pile;
- a player's hand;
- a public meld;
- a temporary command-local selection that has not yet committed;
- an explicitly modelled removed-card zone, if a future variant uses one.

Selections in the browser do not affect this invariant because they are presentation state only.

### 7.2 Turn-stage invariant

Examples:

- `AwaitingDraw`: no meld, lay-off or discard command is accepted.
- `AfterDraw`: draw is no longer accepted; meld/lay-off/discard may be accepted.
- after discard: the next seat becomes active and stage returns to `AwaitingDraw`.

Command processing must be atomic. A rejected multi-card meld does not remove any submitted card from the hand.

## 8. Commands

Suggested domain commands:

```rust
pub enum GameCommand {
    DrawFromStock,
    DrawFromDiscardTop,
    CreateMeld { cards: Vec<CardId> },
    LayOff { meld_id: MeldId, cards: Vec<CardId> },
    Discard { card: CardId },
}
```

Room-level commands are separate:

```rust
pub enum RoomCommand {
    Join,
    Leave,
    ClaimSeat { seat: SeatIndex },
    SetReady { ready: bool },
    UpdateRules { rules: RulesConfig },
    StartMatch,
    Game(GameCommand),
}
```

Only the room host may update rules or start, subject to server checks. Rules lock when a match starts.

## 9. Events and recipient projections

Canonical domain events describe facts accepted by the server. They may contain private data in protected server storage.

Example:

```rust
pub enum DomainEvent {
    RoundStarted { shuffled_deck_commitment: Option<[u8; 32]> },
    CardsDealt { hands: BTreeMap<PlayerId, Vec<Card>> },
    CardDrawn { player: PlayerId, card: Card, source: DrawSource },
    MeldCreated { player: PlayerId, meld: TableMeld },
    CardsLaidOff { player: PlayerId, meld_id: MeldId, cards: Vec<Card> },
    CardDiscarded { player: PlayerId, card: Card },
    TurnAdvanced { active_player: PlayerId },
    PlayerWentOut { player: PlayerId },
    RoundScored { result: RoundResult },
}
```

Before sending, convert canonical events to a `PlayerEvent` for each recipient.

For `CardDrawn` from stock:

- drawing player sees the card identity;
- opponents see player, source and new hand count;
- spectators see the opponent/public form.

For initial dealing:

- each player receives only their hand;
- all clients receive public hand counts and dealer/turn information.

Do not serialize `DomainEvent` directly onto a client connection.

## 10. Room actor and concurrency

Each active room runs in one Tokio task and receives messages through an `mpsc` channel. This creates an unambiguous total order for room actions.

Processing an accepted command:

1. authenticate the connection;
2. authorize the player's membership and seat;
3. reject an already processed `command_id` by returning its prior receipt;
4. optionally compare `expected_sequence` with the current sequence;
5. invoke `game-core`;
6. assign sequence numbers to emitted events;
7. persist events and command receipt in one transaction;
8. update in-memory room state;
9. create recipient-specific events/snapshots;
10. broadcast;
11. acknowledge the command.

Persistence-before-broadcast prevents clients from observing an event the server cannot recover after a crash. The exact boundary should be benchmarked, but correctness takes priority at card-game traffic levels.

## 11. Networking and reconnection

### 11.1 Connection model

A WebSocket is associated with:

- authenticated user/guest ID;
- connection ID;
- room ID;
- claimed player seat, if any;
- protocol version;
- last acknowledged sequence.

A player may have multiple connections. Define policy explicitly:

- initially, newest connection becomes controlling;
- older connections become read-only mirrors or are closed;
- command authorization checks the controlling connection generation.

### 11.2 Reconnection flow

1. Client reconnects using its authenticated session.
2. Client sends `Resume { room_id, last_seen_sequence }`.
3. Server verifies membership.
4. If a retained event window can satisfy the request, server sends missing recipient-filtered events.
5. Otherwise server sends a fresh `PlayerSnapshot`.
6. Client replaces local authoritative view and resumes presentation.

A snapshot includes:

- current sequence;
- rules profile/version;
- player identity and seat;
- own hand;
- public hand counts;
- table melds;
- stock count;
- discard top and other visible discard information;
- scores;
- active player and turn stage;
- timer deadline, if active;
- disconnection states that are public.

### 11.3 Idempotency

Every mutating client command has a UUID `command_id`. Retrying a command after a connection loss must not apply it twice.

Keep a bounded in-memory cache and durable record for commands relevant to active/recoverable games.

## 12. Persistence and recovery

Recommended model: authoritative event log plus periodic snapshots.

### 12.1 Events

Append every accepted game event with:

- game ID;
- sequence;
- event type/version;
- canonical payload;
- timestamp;
- causative command ID.

### 12.2 Snapshots

Write a snapshot:

- at round start;
- every N accepted commands, initially 20;
- at round end;
- before orderly room suspension.

Encrypt or otherwise protect snapshots and events at rest because they contain hidden hands and deck order. Database access must be server-only.

### 12.3 Recovery

On process restart:

1. load latest snapshot;
2. replay subsequent events;
3. verify sequence and card-conservation invariants;
4. recreate room actor;
5. mark timers using current server time and configured recovery policy;
6. accept reconnects.

If recovery validation fails, quarantine the game rather than guessing state.

## 13. Randomness and fairness

- Shuffle only on the server.
- Use an unpredictable OS-seeded cryptographic RNG or a CSPRNG seeded from it.
- Never accept client-provided deck order.
- Do not log seed or deck order during an active game.
- Tests use deterministic seeded RNG or fixed decks.

Optional future fairness feature:

- publish a cryptographic commitment to the shuffled deck at round start;
- reveal the salt/deck after the round;
- allow clients to verify that the server did not change the deck mid-round.

This detects server manipulation after the fact but does not prevent a malicious server from choosing favourable initial shuffles. Multi-party commit/reveal is possible later but is unnecessary for a casual first release.

## 14. Lobby and match lifecycle

```text
Created
  ↓
WaitingForPlayers
  ↓ all required players ready
Starting
  ↓
RoundInProgress
  ↓
RoundScoring
  ↓ target not reached
BetweenRounds
  └──────────────→ RoundInProgress
  ↓ target reached
MatchComplete
  ↓
Archived
```

Rules:

- A room may contain 2–8 occupied player seats.
- The host can configure room rules before the first match.
- Joining during a round is disabled initially.
- A disconnected player retains their seat for a grace period.
- Other players see that the seat is disconnected, not private network details.
- Host migration occurs automatically if the host leaves; lowest occupied seat index becomes host.
- Host privileges do not include manipulating cards or state.

## 15. Timers and disconnected players

Initial casual defaults:

- no turn timer in private rooms by default;
- optional 30, 60 or 120 second timer;
- disconnect grace period: 120 seconds;
- timer pauses for a short reconnect window, initially 20 seconds, once per player per round;
- after grace expiry, host may remove the player or a bot may take over in a future release.

For the first implementation, prefer ending/suspending the room over auto-playing strategically significant moves. A simple forced draw-and-discard bot may produce unfair outcomes.

## 16. UI design

### 16.1 Main game table

The player sees:

- their hand along the bottom;
- stock and discard pile centrally;
- public melds in the table area;
- opponents around the remaining edges with name, score, card count and connection state;
- active-turn indication;
- action panel with accessible alternatives to dragging.

For 7–8 players on narrow screens, use a scrollable or paged opponent strip rather than shrinking names and card counts beyond readability.

### 16.2 Card interactions

Required interactions:

- click/tap card to select;
- Shift-click or range selection on desktop where appropriate;
- keyboard navigation through hand;
- “Create meld”, “Lay off”, and “Discard” buttons;
- optional drag-and-drop as an enhancement, not the only path;
- client-side visual preview of candidate melds;
- server rejection displayed without losing selection unnecessarily.

Allow players to reorder their hand locally. Local order is not authoritative and need not be broadcast.

### 16.3 Responsive layout

Desktop:

- oval/rectangular table;
- hand spread across bottom;
- melds use central grid.

Mobile portrait:

- opponents in compact header carousel;
- stock/discard and turn action in upper middle;
- table melds in vertically scrollable area;
- hand in horizontally scrollable tray;
- persistent primary-action bar.

### 16.4 Accessibility

- Every card has a textual accessible name.
- Suit is represented by symbol/text as well as colour.
- Red and black colours pass contrast requirements against card backgrounds.
- Full keyboard operation is supported.
- Focus remains predictable after server updates.
- Reduced-motion mode disables dealing/flying animations.
- Important state changes use an appropriate live region without announcing every decorative movement.

## 17. Security and abuse prevention

### 17.1 Trust boundaries

Untrusted:

- browser state;
- WebSocket messages;
- room codes;
- display names;
- client timestamps;
- client-reported connection state.

Trusted only after validation:

- authenticated server session;
- membership records;
- room actor state;
- persisted canonical events.

### 17.2 Controls

- Per-connection and per-user rate limits.
- Message size limits.
- Strict deserialization and protocol version checks.
- Server-side authorization for every command.
- HTML escaping for player names and chat.
- Secure, HttpOnly session cookies.
- CSRF protection for applicable HTTP mutations.
- Origin checks for WebSocket upgrades.
- No secrets or hidden cards in telemetry.
- Generic public errors; detailed internal errors only in protected logs.

### 17.3 Cheating that can and cannot be prevented

Can prevent:

- playing cards not in one's hand;
- acting out of turn;
- manipulating deck order through the client;
- seeing hidden cards through normal protocol messages;
- applying the same command twice;
- changing rules after match start.

Cannot fully prevent:

- two players sharing information externally;
- screen sharing;
- one person controlling multiple accounts;
- a compromised server operator viewing server-side state.

## 18. Observability

Use structured tracing with identifiers:

- request ID;
- connection ID;
- room ID;
- game ID;
- player ID pseudonymous/internal;
- command ID;
- sequence.

Metrics:

- active rooms and connections;
- reconnect rate;
- rejected commands by category;
- room actor mailbox depth;
- command processing latency;
- database transaction latency;
- snapshot/recovery failures;
- average round duration by player count;
- stock recycle frequency;
- abandonment rate.

Never label metrics with unbounded room/player names.

## 19. Testing strategy

### 19.1 Rules examples

Test at minimum:

- valid three/four-card sets;
- invalid duplicate-suit set under two decks;
- valid low-ace and high-ace runs;
- invalid wraparound run;
- extending runs at either end;
- illegal gaps and mixed suits;
- top-discard pickup restriction;
- inability to discard the same card just taken from discard;
- final-discard going-out requirement;
- scoring with all rank classes;
- stock recycling and blocked round.

### 19.2 Property tests

Generate random legal command sequences and assert:

- card conservation;
- unique card location;
- current player is a valid occupied seat;
- accepted sequence increases monotonically;
- rejected commands preserve state;
- player projections do not leak other hands;
- scoring never counts table meld cards as unmelded.

### 19.3 Simulation tests

Implement simple legal-move bots to run thousands of matches for 2–8 players. Use simulations to find:

- non-terminating states;
- stock exhaustion patterns;
- excessively long rounds;
- unexpected scoring distributions;
- rule interactions missed by example tests.

Bots are test infrastructure first, not a promised product feature.

### 19.4 Network tests

- duplicate command IDs;
- reordered/stale expected sequences;
- disconnect immediately after sending a command;
- disconnect after server commit but before acknowledgement;
- multiple controlling connections;
- reconnect after server restart;
- unauthorized room/seat access;
- malformed and oversized messages.

## 20. Deployment design

### 20.1 Initial deployment

One service instance can host:

- HTTP pages/assets;
- WebSockets;
- all active room actors;
- background snapshot tasks.

PostgreSQL is a separate managed or self-hosted service.

Use a reverse proxy/load balancer that supports WebSocket upgrades and has an idle timeout longer than the heartbeat interval.

### 20.2 Horizontal scaling

Before adding instances, introduce room affinity:

- room registry maps each active room to an owning instance;
- new/reconnecting connections route to the owner, or proxy messages;
- use sticky routing only as an implementation convenience, not the sole source of ownership truth;
- persisted snapshots/events allow ownership transfer after failure.

Redis is not required initially. It may later provide distributed presence, registry leases or pub/sub, but it must not become the authoritative game state.

## 21. Delivery phases

### Phase 1: rules engine

- cards/decks and `BasicRummyV1`;
- 2–8 player state model;
- deterministic dealing;
- draw, meld, lay off and discard;
- going out and scoring;
- unit/property/simulation tests.

### Phase 2: two-player vertical slice

- guest sessions;
- create/join room;
- typed WebSocket;
- room actor;
- simple UI;
- reconnect snapshot;
- secure shuffle.

### Phase 3: multiplayer completeness

- 3–8 seat layouts;
- ready/start flow;
- multi-round matches;
- host migration;
- disconnect grace policy;
- persistence/recovery.

### Phase 4: product quality

- mobile polish;
- accessibility audit;
- animations and sound;
- rules explanation/tutorial;
- metrics and moderation;
- load testing.

### Phase 5: extensions

- additional rules profiles;
- bots;
- public matchmaking;
- desktop/mobile packages;
- deck fairness verification;
- spectator/replay mode.

## 22. Key architectural decisions

### ADR-001: Dioxus Fullstack

Use Dioxus for the responsive web client and full-stack integration because the project benefits from shared Rust types, typed server functions/WebSockets and a possible native-client path.

### ADR-002: Server-authoritative rules

All legal game transitions occur on the server. This is mandatory because hands and deck order are hidden information.

### ADR-003: Pure domain crate

The rules engine is isolated from async runtime, network, database and UI dependencies to maximize determinism, testing and reuse.

### ADR-004: One actor per active room

A room task serializes commands and owns mutable state, avoiding complex lock-based concurrency.

### ADR-005: Recipient-specific projections

Authoritative state and events are never sent directly to clients. Every snapshot/event is filtered for its recipient.

### ADR-006: Versioned rules profile

Saved games refer to a named, versioned rules profile. Behaviour changes create a new version rather than silently altering old matches.

### ADR-007: Event log with snapshots

Accepted events are persisted before broadcast, with periodic snapshots for efficient recovery.

## 23. Open questions

- Confirm the final public name of `BasicRummyV1` and how prominently to describe house rules.
- Retain ten-card hands for 6–8 players or reduce them after simulation/playtesting?
- Allow laying off in the same turn as the first meld?
- Permit rearranging existing melds in any future profile?
- What result should a blocked round produce?
- Are anonymous guest rooms sufficient for launch, or are accounts required?
- Should room owners be able to replace disconnected players with bots?
- Is chat needed, given moderation cost?
- Which additional Rummy variant should be second?

## 24. References

- General Rummy and Basic Rummy overview: https://en.wikipedia.org/wiki/Rummy
- Dioxus Fullstack overview: https://dioxuslabs.com/learn/0.7/essentials/fullstack/
- Dioxus Fullstack project setup: https://dioxuslabs.com/learn/0.7/essentials/fullstack/project_setup/
- Dioxus typed WebSockets: https://dioxuslabs.com/learn/0.7/essentials/fullstack/websockets/
