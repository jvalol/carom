//! A run: the rack, the shooter, what is out, and what it took. See
//! `specs/0001-the-ring.md`.

use blitzkit::physics::{step, Body};
use glam::Vec3;

use crate::ring::{self, MARBLES};
use crate::shot::{self, MARBLE_MASS, SHOOTER_MASS};

/// Where a run has got to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// Pointing, with nothing moving.
    Aiming,
    /// A shot is rolling. Nothing can be aimed until it stops.
    Rolling,
    /// The ring is empty.
    Over,
}

/// The shooter is body zero and the thirteen follow it, so an index into
/// `bodies` is an index into `out` with one subtracted and nothing has to be
/// searched for.
pub const SHOOTER: usize = 0;

pub struct Run {
    pub bodies: Vec<Body>,
    /// Which of the thirteen have left the ring. Only ever goes on: a marble
    /// that rolls back in was out at the moment it crossed and stays out.
    out: [bool; MARBLES],
    shots: u32,
    phase: Phase,
    /// How many steps the shot now rolling has taken.
    rolled: usize,
}

impl Default for Run {
    fn default() -> Self {
        Self::new()
    }
}

impl Run {
    pub fn new() -> Self {
        let mut bodies = vec![shot::marble(ring::shoot_from(), SHOOTER_MASS)];
        bodies.extend(
            ring::cross()
                .into_iter()
                .map(|at| shot::marble(at, MARBLE_MASS)),
        );

        Self {
            bodies,
            out: [false; MARBLES],
            shots: 0,
            phase: Phase::Aiming,
            rolled: 0,
        }
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn shots(&self) -> u32 {
        self.shots
    }

    /// How many of the thirteen are still in the ring.
    pub fn left(&self) -> usize {
        self.out.iter().filter(|out| !**out).count()
    }

    pub fn is_out(&self, marble: usize) -> bool {
        self.out.get(marble).copied().unwrap_or(false)
    }

    pub fn shooter(&self) -> Vec3 {
        self.bodies[SHOOTER].position
    }

    /// Takes the shot. Does nothing while one is already rolling, or after the
    /// ring is empty.
    pub fn shoot(&mut self, way: Vec3, speed: f32) {
        if self.phase != Phase::Aiming {
            return;
        }

        // a click with no hold behind it is not a shot. It used to count as
        // one, roll nowhere, and hand the turn straight back, which from the
        // other side of the screen looks like the game ignoring you.
        if speed < shot::STILL || way.length_squared() < 1e-6 {
            return;
        }

        self.bodies[SHOOTER].velocity = way.normalize_or_zero() * speed;
        self.bodies[SHOOTER].spin = Vec3::ZERO;
        self.shots += 1;
        self.phase = Phase::Rolling;
        self.rolled = 0;
    }

    /// One step of a rolling shot: move everything, write down what left the
    /// ring, and see whether it is over.
    pub fn step(&mut self, dt: f32) {
        if self.phase != Phase::Rolling {
            return;
        }

        step(&mut self.bodies, &[ring::table()], shot::GRAVITY, dt);

        // written down as it happens rather than at the end, or a marble that
        // rolls out and back in would never have been out
        for marble in 0..MARBLES {
            if ring::is_out(self.bodies[marble + 1].position) {
                self.out[marble] = true;
            }
        }

        // a net, not the rule. The physics settles a break in about five
        // seconds and `shot::tests::no_shot_runs_for_ever` holds it to that;
        // this is here so a shot nobody foresaw cannot take the window with it.
        self.rolled += 1;
        if !shot::nothing_is_moving(&self.bodies) && self.rolled < shot::LONGEST {
            return;
        }

        // a shooter that left the ring loses the place it had
        if ring::is_out(self.bodies[SHOOTER].position) {
            self.bodies[SHOOTER].position = ring::shoot_from();
            self.bodies[SHOOTER].velocity = Vec3::ZERO;
            self.bodies[SHOOTER].spin = Vec3::ZERO;
        }

        self.phase = if self.left() == 0 {
            Phase::Over
        } else {
            Phase::Aiming
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::vec3;

    /// Runs a shot out without drawing it, and says how many steps it took.
    fn settle(run: &mut Run) -> usize {
        for taken in 1..=shot::LONGEST {
            run.step(shot::STEP);
            if run.phase() != Phase::Rolling {
                return taken;
            }
        }

        shot::LONGEST
    }

    #[test]
    fn a_new_run_is_a_full_ring() {
        let run = Run::new();

        assert_eq!(run.left(), MARBLES);
        assert_eq!(run.shots(), 0);
        assert_eq!(run.phase(), Phase::Aiming);
        assert_eq!(run.shooter(), ring::shoot_from());
    }

    #[test]
    fn the_score_counts_shots() {
        let mut run = Run::new();

        for taken in 1..=3 {
            if run.phase() != Phase::Aiming {
                break;
            }
            run.shoot(shot::aim(run.shooter(), Vec3::ZERO), 4.0);
            settle(&mut run);
            assert_eq!(run.shots(), taken);
        }

        assert!(run.shots() >= 1);
    }

    #[test]
    fn a_shot_with_nothing_behind_it_is_not_a_shot() {
        let mut run = Run::new();

        run.shoot(Vec3::NEG_Z, 0.0);

        assert_eq!(run.shots(), 0, "a click with no hold cost a shot");
        assert_eq!(run.phase(), Phase::Aiming, "it is still your turn");

        run.shoot(Vec3::ZERO, 8.0);

        assert_eq!(run.shots(), 0, "a shot pointing nowhere cost a shot");
    }

    #[test]
    fn a_shot_only_counts_while_aiming() {
        let mut run = Run::new();
        run.shoot(Vec3::NEG_Z, 6.0);

        assert_eq!(run.phase(), Phase::Rolling);
        run.shoot(Vec3::NEG_Z, 6.0);

        assert_eq!(run.shots(), 1, "a second shot went off mid roll");
    }

    #[test]
    fn rolling_back_in_does_not_count() {
        // out is the moment the middle crosses, and nothing takes it back
        let mut run = Run::new();
        run.shoot(Vec3::NEG_Z, 1.0);

        let marble = 0;
        run.bodies[marble + 1].position = vec3(ring::RING_RADIUS + 1.0, 0.5, 0.0);
        run.step(shot::STEP);

        assert!(run.is_out(marble), "it never counted as out");

        run.bodies[marble + 1].position = Vec3::Y * 0.5;
        run.step(shot::STEP);

        assert!(run.is_out(marble), "coming back in unwound it");
    }

    #[test]
    fn an_empty_ring_is_the_end() {
        let mut run = Run::new();
        run.shoot(Vec3::NEG_Z, 1.0);

        for marble in 0..MARBLES {
            run.bodies[marble + 1].position = vec3(ring::RING_RADIUS * 2.0, 0.5, 0.0);
            run.bodies[marble + 1].velocity = Vec3::ZERO;
        }
        run.bodies[SHOOTER].velocity = Vec3::ZERO;
        run.step(shot::STEP);

        assert_eq!(run.left(), 0);
        assert_eq!(run.phase(), Phase::Over);
    }

    #[test]
    fn a_lost_shooter_starts_again_from_the_edge() {
        let mut run = Run::new();
        run.shoot(Vec3::NEG_Z, 1.0);

        run.bodies[SHOOTER].position = vec3(ring::RING_RADIUS * 3.0, 0.5, 0.0);
        run.bodies[SHOOTER].velocity = Vec3::ZERO;
        run.bodies[SHOOTER].spin = Vec3::ZERO;
        for marble in 0..MARBLES {
            run.bodies[marble + 1].velocity = Vec3::ZERO;
            run.bodies[marble + 1].spin = Vec3::ZERO;
        }
        run.step(shot::STEP);

        assert_eq!(run.phase(), Phase::Aiming);
        assert_eq!(run.shooter(), ring::shoot_from());
    }

    #[test]
    fn a_shooter_that_stayed_in_keeps_its_place() {
        let mut run = Run::new();
        run.shoot(Vec3::NEG_Z, 1.0);

        let somewhere = vec3(1.0, ring::MARBLE_RADIUS, 2.0);
        run.bodies[SHOOTER].position = somewhere;
        run.bodies[SHOOTER].velocity = Vec3::ZERO;
        run.bodies[SHOOTER].spin = Vec3::ZERO;
        for marble in 0..MARBLES {
            run.bodies[marble + 1].velocity = Vec3::ZERO;
            run.bodies[marble + 1].spin = Vec3::ZERO;
        }
        run.step(shot::STEP);

        assert_eq!(run.phase(), Phase::Aiming);
        assert_eq!(run.shooter(), somewhere);
    }

    #[test]
    fn a_shot_knocks_something_out() {
        // the whole game in one assertion: aim at the rack, shoot hard, and
        // the ring is emptier than it was
        let mut run = Run::new();
        run.shoot(shot::aim(run.shooter(), Vec3::ZERO), shot::HARDEST);

        let taken = settle(&mut run);

        assert!(taken < shot::LONGEST, "the shot never ended");
        assert!(run.left() < MARBLES, "nothing left the ring");
    }
}
