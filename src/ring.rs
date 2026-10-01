//! The table, the ring drawn on it, and the thirteen in a cross. See
//! `specs/0001-the-ring.md`.

use blitzkit::collision::Aabb;
use glam::{vec2, vec3, Vec3};

/// Every marble is this, the shooter included. A shooter differs by weight
/// rather than by size, which is what a real one does.
pub const MARBLE_RADIUS: f32 = 0.5;

/// How far it is from the middle of the ring to the line.
pub const RING_RADIUS: f32 = 6.0;

/// How far apart the cross sits them, middle to middle.
///
/// Just over touching. Closer and the break is one shove rather than thirteen
/// contacts; further and the arms reach the line, where a tap takes a marble
/// out and the game is over before it has asked anything.
pub const APART: f32 = 1.1;

/// How many marbles in each arm of the cross. Four arms and a middle makes
/// thirteen, which is what a ring is racked with.
pub const ARM: usize = 3;
pub const MARBLES: usize = ARM * 4 + 1;

/// How big the table is, measured out from the middle.
///
/// Wide enough that the hardest shot cannot put a marble over the edge: at
/// `shot::ROLLING` a marble struck at `shot::HARDEST` runs out in about
/// fourteen units, and it starts at most `RING_RADIUS` from the middle. A
/// marble that leaves the table falls for ever, and a body still falling is a
/// body still moving, so the shot would never end.
pub const TABLE_HALF: f32 = 30.0;

/// The table, a solid with its top at y zero.
pub fn table() -> Aabb {
    Aabb::from_center_size(
        vec3(0.0, -1.0, 0.0),
        vec3(TABLE_HALF * 2.0, 2.0, TABLE_HALF * 2.0),
    )
}

/// The thirteen, resting on the table in a cross.
pub fn cross() -> Vec<Vec3> {
    let mut at = vec![vec3(0.0, MARBLE_RADIUS, 0.0)];

    for step in 1..=ARM {
        let out = step as f32 * APART;
        for along in [vec3(out, 0.0, 0.0), vec3(-out, 0.0, 0.0)] {
            at.push(along + Vec3::Y * MARBLE_RADIUS);
        }
        for along in [vec3(0.0, 0.0, out), vec3(0.0, 0.0, -out)] {
            at.push(along + Vec3::Y * MARBLE_RADIUS);
        }
    }

    at
}

/// How far from the middle of the ring something is, across the table.
///
/// Across, not through the air: a marble bouncing is no further out for being
/// off the table, and the ring is a line drawn on a flat thing.
pub fn from_the_middle(at: Vec3) -> f32 {
    vec2(at.x, at.z).length()
}

/// Whether a marble's middle has left the ring.
pub fn is_out(at: Vec3) -> bool {
    from_the_middle(at) > RING_RADIUS
}

/// Where you shoot from when you have no position of your own: the edge.
pub fn shoot_from() -> Vec3 {
    vec3(0.0, MARBLE_RADIUS, RING_RADIUS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cross_fits_in_the_ring() {
        let at = cross();

        assert_eq!(at.len(), MARBLES);

        for one in &at {
            assert!(
                !is_out(*one),
                "a marble starts outside the ring at {:?}",
                one
            );
            assert!(
                from_the_middle(*one) < RING_RADIUS - MARBLE_RADIUS * 2.0,
                "a marble starts within a tap of the line at {:?}",
                one
            );
            assert_eq!(one.y, MARBLE_RADIUS, "a marble starts off the table");
        }

        for (n, one) in at.iter().enumerate() {
            for other in at.iter().skip(n + 1) {
                assert!(
                    one.distance(*other) > MARBLE_RADIUS * 2.0,
                    "two marbles start inside each other, {:?} and {:?}",
                    one,
                    other
                );
            }
        }
    }

    #[test]
    fn out_is_the_middle_leaving() {
        // the middle, not the whole marble: half of it over the line is still
        // in, and a hair past the middle is out
        let just_in = vec3(RING_RADIUS - 1e-3, MARBLE_RADIUS, 0.0);
        let just_out = vec3(RING_RADIUS + 1e-3, MARBLE_RADIUS, 0.0);

        assert!(!is_out(just_in));
        assert!(is_out(just_out));
    }

    #[test]
    fn resting_inside_is_not_out() {
        for at in cross() {
            assert!(!is_out(at));
        }
        assert!(!is_out(shoot_from()));
    }

    #[test]
    fn you_shoot_from_the_line() {
        let from = shoot_from();

        assert!(!is_out(from), "the shooting mark is outside the ring");
        assert!((from_the_middle(from) - RING_RADIUS).abs() < 1e-5);
    }

    #[test]
    fn height_is_not_distance() {
        // a marble in the air over the middle is not out for being in the air
        assert!(!is_out(vec3(0.0, 50.0, 0.0)));
    }
}
