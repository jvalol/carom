# 0001 The ring

**Status:** implemented
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

**A shot** is a direction and a speed. Moving the mouse turns it, and so do the
left and right arrow keys for a finer aim. Holding the left button winds it up
and letting go takes it, with the beads in front of the shooter showing how hard.

That is not quite what this spec first said. It said you set the speed by how far
you drag, which wants the cursor's place on the table, which wants a ray from the
cursor into the world. That is blitzkit spec 0025 and it is still a draft, so the
game would have had to do the unprojection itself, in the wrong place. Holding
reads the same way round and costs nothing.

**Every press is a shot.** A click with no hold behind it sends the softest
there is rather than nothing. A shot of nothing rolls nowhere and hands the turn
straight back, which from the other side of the screen is the game ignoring you,
and that is what it looked like the first time anyone played it. Anything you can
press moves the marble, the way it does with a thumb. A shot pointing nowhere is
still refused, because there is no such aim.

The shot is over when every marble has stopped.

**A marble out of the ring is out of play.** It is won, it stays where it came to
rest, and no later shot moves it. Left in play a later shot walks it further out,
and a few of those put it over the edge of the table, where it falls for ever. A
falling body is a body still moving, so the shot never ends and only the twenty
second net stops it, with the mouse ignoring you for all of it.

**The table is big enough that nothing can reach its edge.** The number comes
from a test that shoots off the line away from the ring at everything there is,
which is the worst a shot can do and the case nothing had tried. It was worked
out by hand first and the hand was wrong by more than a factor of two: a full
power shot runs about 33 units, not 19.

**The longest shot is under three seconds**, and that is the one that hits
nothing at all. It is also the thing the player waits through, so it has a test
of its own rather than being whatever the other numbers happen to leave.

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

**One pass over the pairs holds up.** `physics::step` resolves each pair once per
step, in index order, with no iteration, and thirteen in a tight cross struck at
`HARDEST` is a stack of simultaneous contacts resolved one at a time. It was the
open question when this spec was written and the break answers it: the rack
scatters, nothing ends up inside anything, and every shot settles in about five
seconds. Nothing in the engine needed changing for it.

**But gravity has to stay near what the engine expects.** blitzkit's `SETTLES_AT`
is a speed, 0.6, below which a bounce is dropped. Gravity puts `g * dt` back into
a resting body every step, and at 24 units that is 0.2 a step, a third of the
threshold: a marble came to rest and then sat trembling on the table for ever,
and the shot never ended. At 12 it settles. Twelve is also plenty for a game
played flat, since nothing here is ever more than a marble's width off the table.

That is a real edge of spec 0030 rather than a number this game picked for feel,
and it is written down here because this is where it was found.

## Acceptance criteria

- A shooter striking a marble head on stops and sends it on. — `shot::tests::a_head_on_hit_passes_the_speed_along`
- A cut shot sends the marble along the line between their middles. — `shot::tests::a_marble_leaves_along_the_line_of_middles`
- And the two separate by about a right angle, which equal spheres do. — `shot::tests::they_part_at_a_right_angle`
- A harder shot sends a marble further. — `shot::tests::harder_sends_it_further`
- A marble whose middle leaves the ring is out. — `ring::tests::out_is_the_middle_leaving`
- Height is not distance: a marble in the air over the ring is still in it. — `ring::tests::height_is_not_distance`
- A marble resting inside is not out. — `ring::tests::resting_inside_is_not_out`
- The thirteen start inside the ring, clear of each other and of the line. — `ring::tests::the_cross_fits_in_the_ring`
- You shoot from the line itself. — `ring::tests::you_shoot_from_the_line`
- An aim stays on the table however it was taken. — `shot::tests::aiming_stays_on_the_table`
- A shot ends when everything has stopped. — `shot::tests::a_shot_ends_when_nothing_is_moving`
- And every shot ends, from any aim, at full power, against the whole rack. — `shot::tests::no_shot_runs_for_ever`
- A new run is a full ring and no shots. — `game::tests::a_new_run_is_a_full_ring`
- Out is the moment the middle crosses, and coming back in does not unwind it. — `game::tests::rolling_back_in_does_not_count`
- A shooter that leaves the ring goes back to the edge. — `game::tests::a_lost_shooter_starts_again_from_the_edge`
- One that stayed in keeps its place, which is the whole decision. — `game::tests::a_shooter_that_stayed_in_keeps_its_place`
- Nothing can be shot while a shot is rolling. — `game::tests::a_shot_only_counts_while_aiming`
- A hard shot at the rack empties some of the ring. — `game::tests::a_shot_knocks_something_out`
- A whole run of shots never runs long, and nothing leaves the table. — `game::tests::a_whole_run_never_runs_long`
- Nothing can reach the edge of the table, at any weight. — `shot::tests::nothing_can_reach_the_edge`
- And the shot that hits nothing still ends soon. — `shot::tests::a_shot_that_goes_nowhere_still_ends_soon`
- Clearing the ring ends the game. — `game::tests::an_empty_ring_is_the_end`
- The score is the shots taken. — `game::tests::the_score_counts_shots`
- The shot points where it is aimed, and opens across the ring. — `carom_game::tests::the_shot_points_where_it_is_aimed`
- The mouse turns it, and never off the table. — `carom_game::tests::the_mouse_turns_the_shot`
- Holding the button winds it up, to the hardest and no further. — `carom_game::tests::holding_the_button_builds_the_shot`
- Letting go takes the shot and unwinds it. — `carom_game::tests::letting_go_takes_the_shot`
- A click with no hold behind it still shoots. — `carom_game::tests::a_click_with_no_hold_still_shoots`
- A shot pointing nowhere, or at no speed at all, is refused. — `game::tests::a_shot_with_nothing_behind_it_is_not_a_shot`
- The readout is cleared each frame rather than piling up. — `carom_game::tests::the_readout_does_not_pile_up`
- R racks them again once the ring is empty. — `carom_game::tests::r_racks_them_again`
- One shot reads as one shot, and two as two. — `carom_game::tests::one_shot_is_not_one_shots`
- The readout's lines are evenly spaced. — `carom_game::tests::the_readout_lines_are_evenly_spaced`

### Verified by hand

- The break does not scatter, and that is right. Dead on into the near arm, the
  impulse runs down the column the way a Newton's cradle does: the far marble
  leaves the ring, the ones between it and the shooter barely move, and the
  shooter is left wedged in the rack with a bad next shot. An angled shot is
  what scatters. This spec said "thirteen marbles scatter" before anyone had
  run it, and that was a guess.
- None of them passes through another or ends up inside another.
- The beads in front of the shooter grow as the shot winds up, so how hard it
  will be is something you see rather than something you count.
- A cut shot looks like a cut shot. This is the one that says whether the engine's
  impulse is right, because the right angle between the two paths is something
  anyone who has played knows by eye.
- Marbles roll and slow to a stop rather than coasting on, and they stop in a
  time that feels like a table rather than like ice.
- A marble rolling over the ring line is readable as out at the moment it crosses.

**The readout is cleared every frame.** `blitzkit::start` resets the scene
between frames and does not reset the text, so a game that pushes its readout and
never clears it grows the text buffer by a line a frame. carom ran for a couple
of minutes and wgpu killed it at 2.7 gigabytes. Every other game on the engine
remembers; this one did not, and that is worth writing down because it is a trap
rather than a mistake.

## Out of scope

An opponent, keeps and taking marbles off each other, and every other rule of
real ringer beyond knocking them out. Lagging for first shot. Spin put on the
shooter deliberately, which is what a real player's knuckle does and which spec
0030 has no way to ask for. Boxes, slopes, walls around the ring. Sound.

