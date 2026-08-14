# Rummy implementation plan

## Purpose and planning constraints

This plan takes the repository from its current scaffold to a production-capable online Rummy game. Work is ordered as vertical, testable increments rather than by crate alone.

For the initial implementation:

- the server runs as a single process;
- each active room is owned by one Tokio task and reached through channels;
- rooms, guest identities, command receipts, event history, and reconnect data are held in memory;
- a process restart may discard all rooms and matches;
- PostgreSQL, Redis, durable recovery, and multi-instance routing are explicitly deferred to Phase 7;
- `BasicRummyV1` is the only rules profile required before the first playable prototype;
- the browser remains untrusted, even while all server state is in memory.

Every phase ends with an acceptance gate. A phase is complete only when its listed behavior is implemented and its gate passes.

## Current baseline

The workspace and intended crate boundaries already exist. `game-core` contains card/ID types and a partially implemented `Meld` type with 22 unit tests. Most of `game-core`'s state, command, transition, event, scoring, configuration, and view modules are placeholders. Protocol envelopes have begun, but client messages, server messages, and snapshots are not implemented. The server, room actor, WebSocket path, and UI are currently scaffolds.

At the time this plan was written, `cargo test --workspace --all-features` ran 22 `game-core` tests: 17 passed and 5 failed. The failures cover high-ace run behavior, joker placement, set capacity with jokers, and stable serialization of sets. Although jokers are disabled in `BasicRummyV1`, these failures must be resolved or unsupported joker behavior must be removed from the active API before building state transitions on top of it.

## Phase 0 — Stabilize the foundation

Goal: establish a trustworthy, documented baseline on which the rules engine can be built.

### Steps

1. Reconcile `docs/design.md` with the near-term scope.
   - Record that persistence/recovery references describe the later target architecture, not the initial implementation.
   - Confirm the `BasicRummyV1` decisions needed by the engine: ace-low/ace-high behavior, no jokers, ten-card hands, top-discard-only pickup, final discard required, and two stock recycles.
   - Add a short decision for laying off after the first meld; if product input is unavailable, use the design's current proposed behavior and keep it configurable.

2. Make the existing meld implementation deterministic and profile-driven.
   - Fix ace boundary handling and prevent `K-A-2` wrapping.
   - Replace unordered serialized collections with deterministic representations where they enter equality, snapshots, or protocol data.
   - Either correctly support joker validation behind configuration or explicitly reject jokers in `BasicRummyV1`; do not leave contradictory behavior.
   - Expose read-only meld data needed by transitions and views without allowing callers to violate invariants.

3. Establish reusable deterministic test fixtures in `test-support`.
   - Add fixed card/card-ID builders, ordered decks, player builders, and seeded shuffle helpers.
   - Ensure fixtures do not depend on wall-clock time or random UUID values when assertions require stable output.

4. Make the workspace quality commands reliable.
   - Resolve all existing test failures.
   - Address warnings owned by this workspace; separately record unavoidable upstream future-incompatibility warnings.

### Acceptance gate

- Meld tests cover valid sets, valid low/high ace runs, invalid wraparound runs, duplicate physical cards, duplicate-suit sets, and the no-joker Basic profile.
- Repeated serialization of equivalent public values is byte-for-byte stable where protocol snapshots require it.
- `cargo fmt --all -- --check`, `cargo check --workspace --all-targets --all-features`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `cargo test --workspace --all-features` pass.

## Phase 1 — Complete the deterministic `BasicRummyV1` engine

Goal: play and score complete rounds without networking, async code, UI code, or persistence.

### Steps

1. Define explicit rules and domain state.
   - Implement `RulesConfig` plus a versioned `BasicRummyV1` constructor.
   - Define game/round phases, seats, active player, turn stage, hands, stock, discard pile, table melds, recycle count, and match scores.
   - Validate 2–8 distinct seated players and generate the correct physical-card multiset for one or two decks.

2. Implement deterministic round setup.
   - Accept an already shuffled deck (or injected RNG) at the domain boundary.
   - Deal ten cards per player, establish dealer/active seat, seed the discard pile, and retain the remaining stock.
   - Reject malformed supplied decks before mutating state.

3. Define intention-oriented commands, errors, events, and atomic transitions.
   - Implement draw from stock, draw from discard top, create meld, lay off, and discard.
   - Validate actor, active turn, turn stage, ownership, card identity, selected meld, and discard restrictions.
   - Return the next state and accepted domain events; a rejected command must leave the input state unchanged.

4. Implement stock exhaustion and round termination.
   - Preserve the discard top, recycle and reshuffle the remaining discard pile using supplied randomness, and enforce the configured recycle limit.
   - Require a final discard to go out.
   - End and score a blocked round according to the documented Basic profile.

5. Implement scoring and multi-round match state.
   - Score A as 1, numbered cards at face value, and face cards as 10.
   - Award unmelded opponent points to the player who goes out.
   - Rotate dealer/start positions and end the match at the configured target.

6. Implement recipient-specific projections.
   - Create separate authoritative, player, and public/spectator-safe types.
   - Include a player's own hand, opponents' hand counts, public melds, stock count, discard information, scores, active player, and turn stage.
   - Ensure neither serialization nor debug-facing projection data contains another player's hand or stock order.

7. Add broad rules verification.
   - Unit-test every command in every valid/invalid turn stage and all rules listed in `docs/design.md` section 19.1.
   - Add property tests for card conservation, unique card location, rejected-command immutability, legal active seat, deck multiset correctness, and view secrecy.
   - Add deterministic bot/simulation tests that complete many two-player rounds and detect deadlocks or non-termination.

### Acceptance gate

- A deterministic test can set up two players, play legal commands through a complete round, score it, and start the next round using only `game-core` APIs.
- The same initial state plus command sequence produces the same transitions and events.
- All rules unit/property/simulation tests pass, including card conservation and hidden-information checks after every accepted command.
- `game-core` has no dependency on Dioxus, Axum, Tokio, SQLx, system time, or browser APIs.

## Phase 2 — Build the in-memory authoritative server slice

Goal: expose the engine safely through one single-process server before investing in the full game UI.

### Steps

1. Define a versioned protocol.
   - Add stable tagged client messages for create room, join room, ready/start, game intentions, heartbeat, and resume.
   - Add command IDs, expected room sequence, acknowledgements, public errors, recipient-filtered events, and player snapshots.
   - Reject unsupported protocol versions and malformed messages without affecting room state.

2. Implement temporary guest identity and room membership.
   - Issue opaque guest/session identifiers and reconnect credentials appropriate for the prototype.
   - Treat room codes only as discovery, never authorization.
   - Authorize every start/game/resume command against the server-owned seat.

3. Implement an in-memory room registry.
   - Create collision-resistant room IDs/codes and channel handles.
   - Store handles in process memory while keeping mutable game state inside the owning actor.
   - Remove abandoned/completed rooms with an explicit lifecycle policy.

4. Implement one actor per room.
   - Serialize join, ready, start, game, disconnect, reconnect, and shutdown inputs through one Tokio channel.
   - Keep player seats, canonical game state, monotonically increasing room sequence, and a bounded command-receipt cache inside the actor.
   - Make duplicate command IDs idempotent and reject stale expected sequences with a recoverable response.

5. Implement secure server-side setup and recipient filtering.
   - Shuffle with OS-seeded server randomness in production and injected/fixed randomness in tests.
   - Convert authoritative transitions to a separate message for each connected recipient.
   - Never send or log another player's hand, the stock order, session credentials, or full canonical events.

6. Implement HTTP/WebSocket lifecycle.
   - Start Axum/Dioxus on configured addresses with tracing and graceful shutdown.
   - Add room creation/join endpoints as appropriate and an authenticated WebSocket upgrade with origin and message-size checks.
   - Route parsed messages to room handles; bound queues and rate/message frequency at the connection edge.
   - Add heartbeat/connection cleanup without using timing sleeps in deterministic room tests.

7. Implement in-memory reconnection.
   - Retain seat ownership and a bounded recipient-safe event window after disconnect.
   - Resume from missing events when possible; otherwise send a fresh player-specific snapshot.
   - Define newest-connection-wins behavior for two controlling connections.

### Acceptance gate

- Integration tests create a room, join exactly two guests, ready/start, and exchange legal game commands over typed WebSockets.
- Tests prove duplicate commands apply at most once, stale sequences cannot roll state backward, and unauthorized clients cannot act for a seat.
- Snapshot/event tests inspect serialized payloads and confirm neither player receives the other's cards or the stock order.
- A disconnect/reconnect test restores the correct seat and view while the same server process remains alive.

## Phase 3 — Deliver the initial two-player prototype

Goal: two people can complete a full `BasicRummyV1` round in separate browsers with clear, accessible controls. Completion of this phase is the concrete initial two-player prototype milestone.

### Steps

1. Build the create/join flow.
   - Implement the home/lobby routes, guest display-name handling, room-code/link sharing, two visible seats, ready state, and host start control.
   - Present server errors and retry paths without inventing local room authority.

2. Build recipient-filtered client state and WebSocket handling.
   - Reduce server snapshots/events into a client view model.
   - Track connecting, connected, reconnecting, and disconnected states.
   - Reconcile all optimistic selection/animation state with authoritative sequence updates.

3. Build the playable table.
   - Render the player's hand, opponent card count, stock count, discard top, public melds, active player, stage, and scores.
   - Support stock/discard draw, multi-card selection, create meld, lay off, and discard.
   - Disable clearly unavailable controls while still relying on server validation.

4. Meet minimum responsive and accessible interaction requirements.
   - Support mouse, touch, and full keyboard play without requiring drag-and-drop.
   - Give every card an accessible rank/suit name, do not convey suit by color alone, and provide visible focus styles.
   - Add live feedback for accepted/rejected actions and connection changes; respect reduced-motion preferences.
   - Verify desktop and mobile portrait layouts at representative viewport sizes.

5. Make round completion understandable.
   - Show who went out, points awarded, current match score, and a clear return/rematch path.
   - Surface the `BasicRummyV1` house rules used by the prototype.

6. Add end-to-end coverage.
   - Drive two independent browser contexts through create, join, ready, start, draw, meld/lay-off where the fixed deck permits, discard, reconnect, and round completion.
   - Use a deterministic test-only deck hook that cannot be selected by production clients.
   - Test an attempted out-of-turn move and verify the UI recovers from the server rejection.

### Acceptance gate — two-player prototype complete

- From a clean server start, two browser users can create/join a private room and finish a complete Basic Rummy round without manual server intervention.
- Refreshing or briefly disconnecting either browser restores that player's correct hand and current public state while the process remains alive.
- Neither browser payload/DOM/debug output exposes the opponent's cards or stock order.
- The critical path passes automated domain, server integration, and two-browser end-to-end tests.
- A short manual checklist passes on desktop and mobile viewport sizes using keyboard, mouse, and touch-equivalent controls.
- The prototype documentation explicitly states that rooms are lost on server restart.

## Phase 4 — Harden the in-memory game and match experience

Goal: turn the prototype into a robust single-process two-player experience before expanding player counts.

### Steps

1. Complete multi-round match flow to the target score, including dealer rotation, between-round readiness, rematch, and match-complete screens.
2. Define and implement disconnect grace behavior, host departure/host migration, room cleanup, and safe behavior when a player abandons an active match.
3. Harden command idempotency, bounded event retention, queue backpressure, oversized/malformed payload handling, rate limiting, origin checks, and reconnect-token rotation/revocation.
4. Add structured tracing and low-cardinality metrics for room count, connections, reconnects, command rejection categories, mailbox depth, and command latency without hidden data.
5. Add failure-path tests for disconnect-before-ack, duplicate connections, queue saturation, malformed frames, and shutdown with active rooms.
6. Load-test expected concurrent single-process room counts and record capacity assumptions and operational limits.

### Acceptance gate

- A deterministic integration test completes a multi-round match and rematch.
- Network/security tests cover the failure paths above and find no duplicate state application or secret leakage.
- Load tests meet a documented initial concurrency target with bounded memory/mailboxes and acceptable command latency.
- Graceful shutdown closes connections and actors cleanly; loss of rooms after process exit remains expected and documented.

## Phase 5 — Expand from two players to 3–8 players

Goal: support the full advertised player-count range without weakening rules, secrecy, or usability.

### Steps

1. Generalize lobby seats, readiness, start constraints, turn rotation, dealer rotation, and host migration for 3–8 occupied seats.
2. Verify one-deck behavior for 2–3 players and two-deck physical card identities for 4–8 players, including duplicate-suit set rejection.
3. Add responsive opponent layouts, including a compact carousel/strip on narrow screens, while retaining readable card counts, active-turn state, score, and connection state.
4. Run deterministic simulations for every player count to measure round length, stock recycling, blocked rounds, and ten-card hand viability; use results to resolve the open balance questions before changing `BasicRummyV1`.
5. Add integration and end-to-end scenarios for 3, 4, and 8 players, including middle-seat disconnect/reconnect and turn advancement around disconnected seats according to policy.

### Acceptance gate

- Automated create → join → start → complete-round scenarios pass for every player count from 2 through 8 at the domain/server level.
- Representative 3-, 4-, and 8-player browser tests pass without hidden-card leakage.
- Simulation results and any resulting rules decisions are checked into documentation; behavior changes create a new profile version when compatibility requires it.
- Desktop and mobile layouts remain operable and readable with eight seats.

## Phase 6 — Product quality and release readiness

Goal: make the single-process, in-memory version suitable for a controlled public playtest.

### Steps

1. Conduct an accessibility audit covering keyboard order, screen-reader announcements, focus restoration, contrast, suit differentiation, reduced motion, zoom, and touch target size; add regression tests where practical.
2. Polish responsive layouts, reconnect/error messaging, rules explanations, onboarding, empty/loading states, and action feedback.
3. Add privacy-safe operational dashboards/alerts and a support-friendly room diagnostic view that contains no hands, stock order, credentials, email addresses, or full WebSocket payloads.
4. Perform dependency/security review, fuzz protocol decoding and command validation, and exercise abuse controls.
5. Document local development, deployment, configuration, backup limitations, known rule choices, and the explicit restart-loss limitation.
6. Run a staged playtest and record rule-balance, UX, reconnect, latency, and abandonment findings before declaring the release candidate.

### Acceptance gate

- The full quality command suite and relevant browser/end-to-end/security/load tests pass in CI.
- No critical/high accessibility or security finding remains unresolved.
- Operational runbooks clearly state single-instance requirements and the consequences of restart/failure.
- Playtest findings have owners or explicit deferral decisions.

## Phase 7 — Add durable storage and distributed infrastructure later

Goal: add PostgreSQL and, only where justified, Redis/multi-instance coordination without moving game legality out of the room actor.

This phase starts only after the in-memory model and event boundaries have proven stable. It is not required for the initial two-player prototype or the first single-process playtest.

### Steps

1. Design and migrate PostgreSQL storage for identities/sessions, rooms/membership, append-only canonical events, command receipts, snapshots, and match results.
2. Introduce repository/event-store interfaces at the server boundary and retain an in-memory implementation for fast tests; do not add storage dependencies to `game-core`.
3. Persist accepted events and command receipts transactionally before broadcast, protect hidden state at rest, and write periodic/versioned snapshots.
4. Recover room actors by loading a snapshot and replaying later events, then validate sequence and card-conservation invariants before allowing reconnects.
5. Add restart tests for accepted-command-before-ack, command idempotency across restart, corrupt recovery quarantine, and session/seat restoration.
6. Establish explicit retention, archival, deletion, migration/versioning, backup, restore, and encryption policies.
7. Introduce Redis only for a demonstrated distributed need such as instance/room leases, presence, routing metadata, or pub/sub. Redis must not become the authoritative game state.
8. Before running multiple server instances, implement room affinity/ownership transfer, lease fencing, reconnect routing, and failure tests that prove only one actor owns a room at a time.

### Acceptance gate

- A process restart restores active rooms, seats, sequence numbers, hidden hands, command idempotency, and reconnect behavior from PostgreSQL.
- Recovery rejects/quarantines inconsistent state rather than guessing.
- Multi-instance tests, if Redis/routing is introduced, prove single room ownership during normal operation and failover.
- Database/Redis outages have documented behavior and cannot cause divergent accepted game state or secret leakage.

## Phase ordering and change discipline

- Do not begin UI legality logic to compensate for an incomplete `game-core`; the UI may preview but the server decides.
- Do not serialize `GameState` or canonical private events to clients as a shortcut.
- Keep persistence abstractions out until Phase 7 unless a narrow interface is needed to avoid coupling; the initial source of truth is the in-memory room actor.
- Resolve a phase's acceptance gate before depending on it in the next phase. Small exploratory spikes are allowed, but production code should follow the boundaries above.
- Any change to rules behavior needs the lowest-layer unit/property tests and an explicit `RulesConfig`/profile decision.
- At every phase boundary run:

  ```bash
  cargo fmt --all -- --check
  cargo check --workspace --all-targets --all-features
  cargo clippy --workspace --all-targets --all-features -- -D warnings
  cargo test --workspace --all-features
  ```

  Also run the server integration, browser/end-to-end, security, database, or load tests introduced by that phase.
