## Findings

1. High: shuffling is currently unreachable. [`initialize_game_from_config()`](/Users/gautham/dev/rummy/crates/game-core/src/state.rs:56) never changes the phase from `WaitingForPlayers` to `InitializingGame`, while [`shuffle_draw_stack()`](/Users/gautham/dev/rummy/crates/game-core/src/state.rs:84) only accepts `InitializingGame` or `RestockingDrawStack`. Both permitted phases are private and currently cannot be entered.

2. High: [`TurnState::Drawn { drawn }`](/Users/gautham/dev/rummy/crates/game-core/src/state.rs:11) cannot distinguish stock and discard draws. The stored card would therefore either:
   - incorrectly prevent rediscarding a stock-drawn card; or
   - fail to enforce the restriction for discard draws.

   Prefer:

   ```rust
   pub enum TurnState {
       AwaitingDraw,
       AfterDraw {
           forbidden_discard: Option<CardId>,
       },
   }
   ```

3. High: deck construction remains nondeterministic. [`initialize_deck()`](/Users/gautham/dev/rummy/crates/game-core/src/state.rs:65) calls `CardId::new()` internally. The new explicit-ID card constructors are good, but the IDs must be supplied to deck generation or generated through an injected source.

4. High: player count is not validated against actual players. [`GameConfig::player_count()`](/Users/gautham/dev/rummy/crates/game-core/src/config.rs:160) may say four while `GameState.players` contains zero, two, duplicates, or eight. Initialization should validate the ordered player collection before committing configuration/deck state.

5. Medium: `Discarded` should probably not be a persistent turn state. A valid discard atomically ends the turn and advances the active player to `AwaitingDraw`. Exposing `Discarded` creates an intermediate state in which the engine can become stuck.

6. Medium: `RestockingDrawStack` should not be a long-lived game phase. Recycling and shuffling should happen atomically during a draw attempt, using supplied randomness. Otherwise commands can observe a partially restocked round.

7. Medium: [`winning_player_id`](/Users/gautham/dev/rummy/crates/game-core/src/state.rs:27) is premature terminology. At this point the player is the `declarer`; the match winner is not known until scoring updates the accumulated totals.

8. Medium: the state still lacks Step 1 data:
   - recycle count;
   - accepted declaration;
   - opponents’ scoring submissions;
   - match scores;
   - round versus match completion;
   - stable starting/active player tracking between rounds.

9. Medium: [`Player`](/Users/gautham/dev/rummy/crates/game-core/src/player.rs:10) still stores `draft_melds` and `uncategorized_cards`. Those are explicitly client-local presentation state and should not be in authoritative game state.

## Configuration review

`GameConfig` is in much better shape:

- It has a versioned profile.
- Player counts are validated.
- Deck count is derived correctly.
- Fields are private.
- Removing direct `Deserialize` prevents invalid field combinations from bypassing the constructor.
- The canonical Basic Rummy values are represented explicitly.

Minor improvements:

- `basic_rummy_v1()` should return `Result<Self, ConfigError>` rather than the broad `GameResult<Self>`.
- `UnsupportedPlayerCount` should retain the rejected value.
- The test-only “BasicRummyV1 with jokers” configuration technically contradicts its profile identity. A test-specific meld configuration or lower-level `MeldRules` would be cleaner.

## Recommended next shape

```rust
pub enum TurnState {
    AwaitingDraw,
    AfterDraw {
        forbidden_discard: Option<CardId>,
    },
}

pub enum GamePhase {
    WaitingForPlayers,
    Playing {
        active_player_index: usize,
        turn: TurnState,
    },
    Scoring {
        declarer: PlayerId,
    },
    RoundComplete,
    MatchComplete,
}
```

Then make initialization one validated, atomic operation that accepts:

- `GameId`;
- `GameConfig`;
- an ordered list of distinct players;
- deterministic card IDs or a prebuilt ordered deck.

Tests currently pass: 23/23. Clippy still fails with 24 findings, including several in the new state API.