# 0002 A picture of the ring after the shot

**Status:** implemented
**Date:** 2026-11-03

## Goal

The frame carom puts on its readme is held against a picture of what that
frame should be, so a change to the shot, the rack or the table is seen
rather than shipped.

## Why it belongs here

Spec 0001 says where the marbles start and what a shot does to them, and
every test of it is arithmetic: where a marble ended up, how many left the
ring, which way the shot points. None of them can see the table.

The engine's spec 0076 made the rest of it one call. What was eight lines of
offscreen renderer and comparison in each game that wanted a picture is now
the reference, the size and what to draw.

**And the staging was already here.** `--screenshot` takes the shot at full
power, a fifth of a radian off the middle, and settles it for eight seconds,
because an unbroken rack is a photograph of a setup rather than of a game.
That is the frame `refresh-screenshots` photographs. So there is nothing to
stage for this: it holds the picture that is already on show.

## Behavior

**`CaromGame::new` takes `staged` rather than reading the arguments.** A
picture test asks for the photographed state without `--screenshot` in its
own command line. `main` passes what the arguments say, which is the only
place that reads them.

**One `update`, at no time at all.** The pose takes the shot and settles the
marbles on its own fixed step, so a second call would find `shot_taken`
already set and do nothing. The `dt` is nought, so nothing moves between the
staging and the shutter.

**The picture is the 3D.** The engine's harness leaves the overlay empty, so
the readout above the table is not in shot and a reference does not change
when a line of help does.

**It is the engine's size**, 320 by 240, said once in `picture::SIZE` and
read here.

## Acceptance criteria

- The ring after the shot matches its
  picture. — `carom_game::tests::the_ring_after_the_shot_looks_right`

### Verified by hand

- **The picture itself**, which is the point of it. Eleven of the thirteen
  still in, two outside the ring to the upper left, the shooter at rest
  beside the rack: the same state the committed `media/screenshot.png`
  shows, without the window or the readout.
- **A control.** `POSED_OFF` moved from 0.45 to 0.40, a twentieth of a
  radian, and the test failed on 0.7% of its pixels. Put back and compared.

## Out of scope

**The readout.** It is 2D and the engine's harness draws none of it. Spec
0001's own tests count what is left.

**A second picture.** One frame of the one thing carom does is what there is
to hold; a picture per phase of a shot would be a flipbook nobody reads.
