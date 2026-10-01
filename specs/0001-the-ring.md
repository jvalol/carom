# 0001 The ring

**Status:** draft
**Date:** 2026-10-01

## Goal

Thirteen marbles in a ring and one in your hand. Knock them out and they are
yours; miss and your shooter stays where it stopped, which is where you shoot
from next.

The ninth game on blitzkit, and the first where one body meets another. A carom
is a shot where one ball strikes another, which is the whole subject.

## Why this game exists

Spec 0030 gave the engine spheres with mass, bounce, friction and spin, in two
halves: a body against the static world, and a body against another body. marble
uses the first half and only the first half. Nothing in its course moves, so the
ball never meets another ball, and the second half ships untested by anything but
its own unit tests.

This is the game that is nothing but the second half. Every shot is one sphere
striking another, and what the player is deciding is what that collision will do.
A game whose whole subject is a function is how securitysweep was built and it is
the pattern worth repeating: the rule can be checked without a window, and the
check is of the same arithmetic the pixels come from.

**What it decides.** Aim and power, and then where your shooter is left. The
greedy shot takes the marble nearest the edge and sends your shooter out after
it; the careful one takes a marble further in and dies in the middle of the ring,
with everything still to shoot at. That tension is the game. Without it this is
an aiming exercise.

## Behavior

**The ring** is a flat circle on a flat table, `RING_RADIUS` across, drawn rather
than walled: nothing stops a marble leaving. Leaving is the point.

**Thirteen marbles** start in a cross at the middle, the standard ringer layout,
each `MARBLE_RADIUS` and all the same mass. Your shooter is the same size and
heavier, which is what a real shooter is.

**A shot** is a direction and a speed. You aim with the mouse along the table and
set the speed with how far you drag, up to `HARDEST`. The shot is over when every
marble has stopped.

**A marble is out when its middle leaves the ring**, not when it stops outside.
A marble that rolls out and back in is out. The count is yours at that moment and
nothing takes it back.

**Your shooter sticks where it stops.** The next shot starts from there, pointing
where you like. If your shooter leaves the ring you lose your turn and shoot from
the edge again, which costs you the position you had.

**The game is over when the ring is empty**, and your score is the shots it took.
Fewer is better, so the clock is a count rather than a timer. There is no
opponent in this spec.

**A marble at rest stays at rest.** Nothing here is driven, so the only thing
that moves a marble is being hit, and the only thing that stops one is the table.

## What it asks of blitzkit

**Rolling resistance, which is blitzkit spec 0031 and exists because of this.**
Friction in spec 0030 acts on the velocity at the contact point, and a ball
rolling without slipping has none there, so no impulse was applied and it rolled
for ever. marble hid that behind its own coast friction, which is a drive model
rather than a property of the ball, and it hid it well enough that nobody
noticed. A game where everything is coasting cannot hide it: a shot would never
end. So the engine gained a couple against the spin, off by default, and a
marble here asks for it.

**And possibly more than one pass over the pairs.** `physics::step` resolves each
pair once per step, in index order, with no iteration. Two marbles is exact.
Thirteen in a tight cross, struck hard, is a stack of simultaneous contacts
resolved one at a time, and a single pass through them is an approximation that
can leave marbles overlapping or send one through a gap it should not fit
through. Whether it is good enough is a question the break shot answers, and it
is the first thing to look at if the opening shot behaves oddly.

This spec does not assume the answer. If one pass holds up, nothing changes.

## Acceptance criteria

- A shooter striking a marble head on stops and sends it on. — `shot::tests::a_head_on_hit_passes_the_speed_along`
- A cut shot sends the marble along the line between their middles. — `shot::tests::a_marble_leaves_along_the_line_of_middles`
- And the two separate by about a right angle, which equal spheres do. — `shot::tests::they_part_at_a_right_angle`
- A harder shot sends a marble further. — `shot::tests::harder_sends_it_further`
- A marble whose middle leaves the ring is out. — `ring::tests::out_is_the_middle_leaving`
- And stays out if it rolls back in. — `ring::tests::rolling_back_in_does_not_count`
- A marble resting inside is not out. — `ring::tests::resting_inside_is_not_out`
- The thirteen start inside the ring and clear of each other. — `ring::tests::the_cross_fits_in_the_ring`
- A shot ends when everything has stopped. — `shot::tests::a_shot_ends_when_nothing_is_moving`
- And every shot ends, from any legal aim and power. — `shot::tests::no_shot_runs_for_ever`
- A shooter that leaves the ring goes back to the edge. — `ring::tests::a_lost_shooter_starts_again_from_the_edge`
- Clearing the ring ends the game. — `game::tests::an_empty_ring_is_the_end`
- The score is the shots taken. — `game::tests::the_score_counts_shots`

### Verified by hand

- The break reads as a break: thirteen marbles scatter and none of them passes
  through another or ends up inside another.
- A cut shot looks like a cut shot. This is the one that says whether the engine's
  impulse is right, because the right angle between the two paths is something
  anyone who has played knows by eye.
- Marbles roll and slow to a stop rather than coasting on, and they stop in a
  time that feels like a table rather than like ice.
- A marble rolling over the ring line is readable as out at the moment it crosses.

## Out of scope

An opponent, keeps and taking marbles off each other, and every other rule of
real ringer beyond knocking them out. Lagging for first shot. Spin put on the
shooter deliberately, which is what a real player's knuckle does and which spec
0030 has no way to ask for. Boxes, slopes, walls around the ring. Sound.

