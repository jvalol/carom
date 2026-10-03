# carom

A ring of marbles and one to shoot with. The ninth game on `blitzkit`. The
dependency is the published crate, overridden by the engine checkout at
`../blitzkit` when built inside this project folder.

A carom is a shot where one ball strikes another.

## Build and test

Requires Rust 1.87 or newer, the engine's MSRV.

```
cargo build
cargo test
cargo run
cargo clippy
cargo fmt
```

## How work happens here

Behavior changes are spec driven:

1. **Write the spec first.** Copy `specs/TEMPLATE.md` to `specs/NNNN-short-name.md`
   and fill it in. Say what the game does, not how the code does it.
2. **Make the acceptance criteria testable.** Each one names the test that proves
   it, or goes under "Verified by hand" when it needs a window.
3. **Write the tests, then the code.** `cargo test` passes before a commit.
4. **Update the spec when behavior changes.** A spec that disagrees with the game
   is a bug in the spec.

## Layout

- `src/main.rs` — nothing yet. The game is a spec so far.

## Why this game exists

blitzkit spec 0030 gave the engine spheres in two halves: a body against the
static world, and a body against another body. marble uses the first half and
only the first half, because nothing in its course moves and the ball never meets
another ball. The second half ships tested by nothing but its own unit tests.

This game is nothing but the second half. Every shot is one sphere striking
another, and what the player decides is what that collision will do.
securitysweep was built the same way, around three engine functions. The rule
can be checked without a window, against the same arithmetic the pixels come
from.

## Why the shooter sticks where it stops

Without it this is an aiming exercise. With it, the greedy shot takes the marble
nearest the edge and sends your shooter out after it, and the careful one takes a
marble further in and leaves you in the middle with everything still to shoot at.
That tension is the game.

## Why it waited for spec 0031

Spec 0030's friction acts on the velocity at the contact point, and a ball
rolling without slipping has none there, so nothing slowed a roll and a shot
would never have ended. marble hid that behind its own coast friction, which
belongs to its drive model. This game could not borrow it, so the engine gained
rolling resistance, off by default, and a marble here asks for it.
