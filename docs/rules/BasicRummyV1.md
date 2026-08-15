# BasicRummyV1 rules

## 1. Status and scope

`BasicRummyV1` is the rules profile for the first playable release. This file is
the canonical definition of that profile. Changing any behavior recorded here
requires a new versioned profile unless the change only clarifies wording
without changing legal play or scoring.

The profile supports private games for 2–8 players. The server is authoritative
for cards, turns, declaration validation and scoring.

## 2. Players, decks and deal

| Players | Cards per player | Standard decks |
|---:|---:|---:|
| 2 | 10 | 1 |
| 3 | 10 | 1 |
| 4 | 10 | 2 |
| 5 | 10 | 2 |
| 6 | 10 | 2 |
| 7 | 10 | 2 |
| 8 | 10 | 2 |

- Each deck contains the standard 52 cards.
- Jokers are not used.
- Identical-looking cards from different decks are distinct physical cards.
- After dealing, one card starts the face-up discard pile and the remaining
  cards form the face-down stock.

## 3. Melds

A meld is either a set or a run:

- A **set** contains at least three cards of the same rank.
- A set may contain at most one card of each suit. With two decks, two
  identical-looking cards therefore cannot be in the same set.
- A **run** contains at least three consecutive ranks of the same suit.
- A run may contain at most one physical copy of each rank-and-suit position.
- Both ace-low (`A-2-3`) and ace-high (`Q-K-A`) runs are legal.
- Within one run, an ace occupies one end only. Runs cannot wrap (`K-A-2`) and
  one ace cannot act as both low and high.

## 4. Private hand organization

Players may reorder, group and ungroup their own cards at any time, including
during another player's turn. These groups are private, client-local drafts:

- opponents cannot see them;
- the server does not treat them as authoritative melds;
- forming or changing them is not a game action; and
- there are no shared or face-up table melds and no laying off.

A player may retain any cards or candidate melds in their hand during a normal
turn.

## 5. Turn sequence

Only the active player may take game actions. A normal turn is:

1. Draw either the top stock card or the top discard card.
2. Discard one card and pass the turn to the next player.

After drawing, the player may instead declare a complete hand as described in
section 7.

## 6. Drawing and discarding

- Only the top card of the discard pile may be drawn.
- A card drawn from the discard pile becomes part of the player's hand.
- The exact physical card just drawn from the discard pile cannot immediately
  be discarded on the same turn.
- A normal discard is public and ends the player's turn.

## 7. Declaring a complete hand

After drawing, the active player may submit a complete-hand declaration. The
submission contains:

- exactly one card to discard; and
- a partition of every other card in the player's post-draw hand into valid
  melds.

Every authoritative hand card must be accounted for exactly once. A card cannot
be omitted, duplicated, used in multiple melds or used as both the discard and a
meld card. The normal discard restrictions also apply.

The server validates the entire declaration atomically:

- if valid, it commits the discard, accepts the declared melds, ends normal
  turns and starts scoring declarations;
- if invalid, it commits nothing, reveals no draft groups, and leaves the player
  in the after-draw stage so they may revise the declaration or make a normal
  discard.

## 8. Scoring declarations

After a complete-hand declaration is accepted, every other player submits:

- zero or more valid melds; and
- all remaining unmatched cards.

The server validates each submission against that player's authoritative hand.
Every card must appear exactly once across the submitted melds and unmatched
cards. Invalid submissions are rejected and must be corrected.

Only accepted completion and scoring declarations reveal melds. Private draft
groups are never revealed.

## 9. Scoring and matches

Unmatched cards have these values:

- Ace: 1 point;
- 2–10: face value;
- Jack, Queen and King: 10 points.

Cards in a valid submitted meld score zero. The player who declared the complete
hand receives the total value of all opponents' unmatched cards.

A match ends when a player reaches 100 points. If a blocked-round result leaves
multiple players tied at or above the target, the highest total wins and an
equal highest total produces a shared result.

## 10. Stock exhaustion

When the stock is empty:

1. Keep the top discard card in place.
2. Move the rest of the discard pile into a new stock.
3. Shuffle it using secure server-side randomness.
4. Continue the round.

The stock may be recycled at most twice. If nobody successfully declares a
complete hand after the second recycle is exhausted, the round is blocked.

The precise blocked-round award is not yet finalized. It remains a documented
product decision and must be resolved before blocked-round scoring is
implemented.

## 11. Pending operational policy

The legal card rules above are fixed for `BasicRummyV1`. The following
operational policy remains to be selected:

- the deadline for opponents to submit scoring declarations; and
- the deterministic scoring fallback when an opponent disconnects or does not
  submit before that deadline.
