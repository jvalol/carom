//! Aiming, power, and running a shot until everything has stopped. See
//! `specs/0001-the-ring.md`.

use blitzkit::physics::Body;
use glam::{vec3, Vec3};

use crate::ring::MARBLE_RADIUS;

/// How hard the hardest shot is, in units a second.
pub const HARDEST: f32 = 14.0;

/// How slowly everything has to be going before a shot is over.
///
/// Not zero: rolling resistance brings a marble to a dead stop but the last of
/// a bounce can take longer than anyone wants to watch.
pub const STILL: f32 = 0.08;

/// What the physics runs at while a shot is rolling, whatever the frame rate.
///
/// A fixed step so the same shot plays the same on any machine, which is the
/// determinism blitzkit spec 0030 offers and the only reason a test can assert
/// on a number rather than a range.
pub const STEP: f32 = 1.0 / 120.0;

/// How many steps a shot may take before something is wrong. Twenty seconds.
pub const LONGEST: usize = (20.0 / STEP) as usize;

/// What a marble is made of: glass on wood.
pub const BOUNCE: f32 = 0.8;
pub const GRIP: f32 = 0.4;
pub const ROLLING: f32 = 0.24;

/// What they weigh. A shooter is the same size and heavier, as a real one is.
pub const MARBLE_MASS: f32 = 1.0;
pub const SHOOTER_MASS: f32 = 1.6;

/// Down, and how hard.
pub const GRAVITY: Vec3 = vec3(0.0, -12.0, 0.0);

/// A marble resting at `at`.
pub fn marble(at: Vec3, mass: f32) -> Body {
    Body::new(at, MARBLE_RADIUS, mass)
        .with_restitution(BOUNCE)
        .with_friction(GRIP)
        .with_rolling(ROLLING)
}

/// Where you are pointing, flattened onto the table and normalised.
///
/// The y of either end is thrown away rather than carried, so a shot never
/// leaves the table however the aim was taken.
pub fn aim(from: Vec3, at: Vec3) -> Vec3 {
    vec3(at.x - from.x, 0.0, at.z - from.z).normalize_or_zero()
}

/// Whether everything has stopped.
pub fn nothing_is_moving(bodies: &[Body]) -> bool {
    bodies.iter().all(|body| body.velocity.length() < STILL)
}

#[cfg(test)]
mod tests {
    use super::*;
    use blitzkit::physics::step;

    use crate::ring::table;

    fn at(x: f32, z: f32) -> Vec3 {
        vec3(x, MARBLE_RADIUS, z)
    }

    /// Runs until nothing is moving, and says how many steps that took.
    fn settle(bodies: &mut [Body]) -> usize {
        for taken in 1..=LONGEST {
            step(bodies, &[table()], GRAVITY, STEP);
            if nothing_is_moving(bodies) {
                return taken;
            }
        }

        LONGEST
    }

    #[test]
    fn a_head_on_hit_passes_the_speed_along() {
        // equal weights, straight on: the one that was moving is left and the
        // one that was still goes on. Not all of it, because the marbles are
        // not perfectly elastic and the table takes its share.
        let mut bodies = [
            marble(at(-3.0, 0.0), MARBLE_MASS).with_velocity(vec3(6.0, 0.0, 0.0)),
            marble(at(0.0, 0.0), MARBLE_MASS),
        ];

        settle(&mut bodies);

        assert!(
            bodies[1].position.x > 0.5,
            "the still one barely moved, to {}",
            bodies[1].position.x
        );
        assert!(
            bodies[0].position.x < bodies[1].position.x,
            "the shooter passed through, {} against {}",
            bodies[0].position.x,
            bodies[1].position.x
        );
    }

    #[test]
    fn a_marble_leaves_along_the_line_of_middles() {
        // the impulse between two spheres runs through both their middles and
        // nowhere else, so a marble struck off centre goes that way and not
        // the way the shooter was going
        let target = at(0.0, 0.0);
        let mut bodies = [
            marble(at(-3.0, -0.45), MARBLE_MASS).with_velocity(vec3(6.0, 0.0, 0.0)),
            marble(target, MARBLE_MASS),
        ];
        let struck_from = bodies[0].position;

        settle(&mut bodies);

        let went = (bodies[1].position - target).normalize_or_zero();
        let line = (target - struck_from).normalize_or_zero();

        assert!(
            went.dot(line) > 0.9,
            "it left along {:?}, not {:?}",
            went,
            line
        );
    }

    #[test]
    fn they_part_at_a_right_angle() {
        // what equal spheres do, and the thing anyone who has played knows by
        // eye. Equal: a heavier shooter carries on through and the angle
        // closes, which is why both of these weigh the same.
        let mut bodies = [
            marble(at(-3.0, -0.4), MARBLE_MASS).with_velocity(vec3(7.0, 0.0, 0.0)),
            marble(at(0.0, 0.0), MARBLE_MASS),
        ];

        // read just after they touch, not after they have stopped: the table
        // bends both paths on the way out and the angle is about the impulse
        for _ in 0..LONGEST {
            step(&mut bodies, &[table()], GRAVITY, STEP);
            if bodies[1].velocity.length() > 0.5 {
                break;
            }
        }

        let went: Vec<Vec3> = bodies
            .iter()
            .map(|body| vec3(body.velocity.x, 0.0, body.velocity.z).normalize_or_zero())
            .collect();

        let between = went[0].dot(went[1]).clamp(-1.0, 1.0).acos().to_degrees();

        assert!(
            (between - 90.0).abs() < 25.0,
            "they parted at {} degrees",
            between
        );
    }

    #[test]
    fn harder_sends_it_further() {
        let mut gently = [
            marble(at(-3.0, 0.0), SHOOTER_MASS).with_velocity(vec3(4.0, 0.0, 0.0)),
            marble(at(0.0, 0.0), MARBLE_MASS),
        ];
        let mut hard = [
            marble(at(-3.0, 0.0), SHOOTER_MASS).with_velocity(vec3(12.0, 0.0, 0.0)),
            marble(at(0.0, 0.0), MARBLE_MASS),
        ];

        settle(&mut gently);
        settle(&mut hard);

        assert!(
            hard[1].position.x > gently[1].position.x + 1.0,
            "hard {} gently {}",
            hard[1].position.x,
            gently[1].position.x
        );
    }

    #[test]
    fn a_shot_ends_when_nothing_is_moving() {
        let mut bodies = [marble(at(0.0, 0.0), MARBLE_MASS).with_velocity(vec3(5.0, 0.0, 0.0))];

        assert!(!nothing_is_moving(&bodies), "it has not been shot yet");

        let taken = settle(&mut bodies);

        assert!(taken < LONGEST, "it never stopped");
        assert!(nothing_is_moving(&bodies));
    }

    #[test]
    fn no_shot_runs_for_ever() {
        // every legal aim at full power, against the whole rack
        for turn in 0..16 {
            let about = turn as f32 / 16.0 * std::f32::consts::TAU;
            let from = vec3(
                about.sin() * crate::ring::RING_RADIUS,
                MARBLE_RADIUS,
                about.cos() * crate::ring::RING_RADIUS,
            );

            let mut bodies =
                vec![marble(from, SHOOTER_MASS).with_velocity(aim(from, Vec3::ZERO) * HARDEST)];
            bodies.extend(
                crate::ring::cross()
                    .into_iter()
                    .map(|a| marble(a, MARBLE_MASS)),
            );

            let taken = settle(&mut bodies);

            assert!(taken < LONGEST, "a shot from {:?} never ended", from);
        }
    }

    #[test]
    fn aiming_stays_on_the_table() {
        let from = vec3(0.0, MARBLE_RADIUS, 6.0);
        let way = aim(from, vec3(0.0, 40.0, 0.0));

        assert_eq!(way.y, 0.0);
        assert!((way.length() - 1.0).abs() < 1e-5);
        assert!(way.z < 0.0, "it points away from the middle");
    }
}
