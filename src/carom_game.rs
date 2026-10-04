//! The window: the table, the ring, the thirteen, and taking a shot. See
//! `specs/0001-the-ring.md`.

use blitzkit::camera::Camera;
use blitzkit::geometry::Geometry;
use blitzkit::keyboard::{KeyboardInput, KeyboardKey, KeyboardKeyState};
use blitzkit::mesh::{MeshData, Transform};
use blitzkit::mouse::{MouseButton, MouseInput};
use blitzkit::renderer::render_text::{RenderText, TextRenderer};
use blitzkit::renderer::scene::{MeshId, Scene};
use blitzkit::renderer::Renderer;
use blitzkit::sound::SoundSystem;
use blitzkit::Game;
use glam::{vec2, vec3, vec4, Vec2, Vec3, Vec4};

use crate::game::{Phase, Run};
use crate::ring::{MARBLES, MARBLE_RADIUS, RING_RADIUS, TABLE_HALF};
use crate::shot::{self, HARDEST};

/// How much of the window the readout leaves alone, and how far apart its
/// lines sit. One text a line, so a wrapped line's leading never decides it.
const HUD_LEFT: f32 = 20.0;
const HUD_TOP: f32 = 20.0;
const HUD_SIZE: f32 = 20.0;
const HUD_APART: f32 = HUD_SIZE * 1.6;

/// How fast the mouse and the arrow keys turn the shot.
const TURN_PER_SECOND: f32 = 1.6;

/// How long holding the button takes to reach the hardest shot.
const TO_FULL: f32 = 1.1;

/// Where the camera sits and what it looks at. High and back, because the game
/// is read off the table rather than from the marble's own height.
const EYE: Vec3 = vec3(0.0, 15.0, 15.0);

/// How many flat tiles the ring is drawn out of.
const RING_TILES: usize = 96;
const RING_WIDE: f32 = 0.16;

const TABLE: Vec4 = vec4(0.22, 0.30, 0.22, 1.0);
const LINE: Vec4 = vec4(0.92, 0.94, 0.88, 1.0);
const MARBLE: Vec4 = vec4(0.62, 0.78, 0.92, 1.0);
const GONE: Vec4 = vec4(0.42, 0.46, 0.50, 1.0);
const SHOOTER_COLOR: Vec4 = vec4(0.96, 0.62, 0.20, 1.0);
const PULL: Vec4 = vec4(1.0, 0.95, 0.75, 1.0);

/// Where a cursor ray meets the table the marbles sit on.
///
/// Nothing when the ray runs away from the table or along it, which is a cursor
/// above the horizon. The plane is the marbles' middles rather than the table
/// top, so pointing at a marble aims at the marble rather than at its shadow.
fn on_the_table(camera: &Camera, cursor: Vec2) -> Option<Vec3> {
    let ray = camera.ray_through(cursor);

    if ray.direction.y.abs() < 1e-6 {
        return None;
    }

    let along = (MARBLE_RADIUS - ray.origin.y) / ray.direction.y;

    (along > 0.0).then(|| ray.at(along))
}

/// "1 shot" and "2 shots". A count beside a word that only reads right at one
/// of its values is the kind of thing nobody notices writing and everybody
/// notices reading.
fn shots(taken: u32) -> String {
    if taken == 1 {
        String::from("1 shot")
    } else {
        format!("{} shots", taken)
    }
}

pub struct CaromGame {
    run: Run,
    /// Which way the shot points, in radians about the up axis.
    aim: f32,
    /// How hard it will be, from nothing to `HARDEST`.
    power: f32,
    charging: bool,
    /// Left and right, held.
    turning: [bool; 2],
    /// Where the cursor is, in pixels, while it is over the window.
    pointing: Option<Vec2>,
    sphere: Option<MeshId>,
    tile: Option<MeshId>,
    width: f32,
    quitting: bool,
    /// Whether this run is only here to be photographed, and whether the shot
    /// has been taken. See `refresh-screenshots` in the project above.
    ///
    /// The opening frame is thirteen marbles racked in a cross and nothing
    /// having happened, which is a photograph of a setup. A staged run takes a
    /// shot and lets it settle, which is the game: a scattered ring with a
    /// marble out of it.
    staged: bool,
    shot_taken: bool,
}

impl Default for CaromGame {
    fn default() -> Self {
        Self::new()
    }
}

impl CaromGame {
    pub fn new() -> Self {
        // opening aim: across the ring from where you stand, which is the only
        // shot that is obviously worth taking before you know the rack
        let opening = shot::aim(crate::ring::shoot_from(), Vec3::ZERO);

        Self {
            run: Run::new(),
            aim: opening.x.atan2(opening.z),
            power: 0.0,
            charging: false,
            turning: [false; 2],
            pointing: None,
            sphere: None,
            tile: None,
            width: 800.0,
            quitting: false,
            staged: crate::staged(),
            shot_taken: false,
        }
    }

    /// How hard the staged shot is and how far off the middle it points.
    ///
    /// Dead on into the near arm does not scatter: the impulse runs down the
    /// column like a Newton's cradle, the far marble leaves and the ones
    /// between barely move. An angled shot is what scatters, which is spec
    /// 0001's own hand check and the picture worth taking.
    /// Measured over power and angle rather than picked: at seven tenths and
    /// a fifth of a radian the rack barely moved and all thirteen stayed in.
    /// Full power at 0.45 is the one that scatters and puts two out.
    const POSED_POWER: f32 = HARDEST;
    const POSED_OFF: f32 = 0.45;
    const POSED_SETTLES_FOR: f32 = 8.0;

    /// Takes a shot for the camera and lets it come to rest. See
    /// `refresh-screenshots`.
    ///
    /// On the first frame rather than over six real seconds, because the
    /// shutter is on a timer and will not wait for the marbles to stop.
    fn pose(&mut self) {
        self.shot_taken = true;
        self.aim += Self::POSED_OFF;
        self.run.shoot(self.way(), Self::POSED_POWER);

        let step = 1.0 / 120.0;
        let mut at = 0.0;
        while at < Self::POSED_SETTLES_FOR {
            self.run.step(step);
            at += step;
        }
    }

    /// Which way the shot goes, on the table.
    pub fn way(&self) -> Vec3 {
        vec3(self.aim.sin(), 0.0, self.aim.cos())
    }

    fn line(&self, n: usize) -> Vec2 {
        vec2(HUD_LEFT, HUD_TOP + n as f32 * HUD_APART)
    }

    fn readout(&self) -> Vec<String> {
        match self.run.phase() {
            Phase::Over => vec![
                format!("cleared in {}", shots(self.run.shots())),
                String::from("press r to rack them again"),
            ],
            _ => vec![
                format!(
                    "{} of {} left, {}",
                    self.run.left(),
                    MARBLES,
                    shots(self.run.shots())
                ),
                String::from("Move the mouse or press the left and right arrow keys to aim"),
                String::from("Hold the left mouse button to pull back, let go to shoot"),
            ],
        }
    }
}

impl Game for CaromGame {
    fn initialize(
        &mut self,
        _geometry: &mut Geometry,
        _text_renderer: &mut TextRenderer,
        _sound_system: &SoundSystem,
        window_size: (f32, f32),
    ) {
        self.resized(window_size);
    }

    fn resized(&mut self, window_size: (f32, f32)) {
        self.width = window_size.0;
    }

    fn load(&mut self, renderer: &mut Renderer) {
        self.sphere = Some(renderer.add_mesh(&MeshData::sphere(24, 16)));
        self.tile = Some(renderer.add_mesh(&MeshData::cube()));
    }

    fn update(
        &mut self,
        dt: f32,
        _geometry: &mut Geometry,
        text_renderer: &mut TextRenderer,
        _sound_system: &SoundSystem,
    ) {
        // the engine resets the scene between frames and does not reset this,
        // so a readout pushed and never cleared grows by a line a frame until
        // the text buffer is larger than the device will allocate. carom ran
        // for a couple of minutes and wgpu killed it at 2.7 gigabytes.
        text_renderer.reset();

        if self.staged {
            if !self.shot_taken {
                self.pose();
            }
            for (n, text) in self.readout().into_iter().enumerate() {
                text_renderer.push_render_text(RenderText {
                    position: self.line(n),
                    color: vec4(1.0, 1.0, 1.0, 0.9),
                    size: HUD_SIZE,
                    text,
                    ..Default::default()
                });
            }
            return;
        }

        let turn = (self.turning[1] as i32 - self.turning[0] as i32) as f32;
        self.aim += turn * TURN_PER_SECOND * dt;

        if self.charging {
            self.power = (self.power + HARDEST / TO_FULL * dt).min(HARDEST);
        }

        // a fixed step, so the same shot plays the same whatever the frame rate
        if self.run.phase() == Phase::Rolling {
            let mut left = dt;
            while left > 0.0 {
                self.run.step(shot::STEP);
                left -= shot::STEP;
            }
        }

        for (n, text) in self.readout().into_iter().enumerate() {
            text_renderer.push_render_text(RenderText {
                position: self.line(n),
                color: vec4(1.0, 1.0, 1.0, 0.9),
                size: HUD_SIZE,
                text,
                ..Default::default()
            });
        }
    }

    fn draw(&mut self, scene: &mut Scene, camera: &mut Camera) {
        // the camera first, so the ray the aim is taken from is this frame's
        camera.position = EYE;
        camera.target = Vec3::ZERO;

        if let Some(cursor) = self.pointing {
            if let Some(at) = on_the_table(camera, cursor) {
                let way = shot::aim(self.run.shooter(), at);
                if way != Vec3::ZERO {
                    self.aim = way.x.atan2(way.z);
                }
            }
        }

        let (Some(sphere), Some(tile)) = (self.sphere, self.tile) else {
            return;
        };

        scene.push_colored(
            tile,
            &Transform::at(vec3(0.0, -1.0, 0.0)).with_scale(vec3(
                TABLE_HALF * 2.0,
                2.0,
                TABLE_HALF * 2.0,
            )),
            TABLE,
        );

        for n in 0..RING_TILES {
            let about = n as f32 / RING_TILES as f32 * std::f32::consts::TAU;
            let at = vec3(about.sin(), 0.0, about.cos()) * RING_RADIUS;

            scene.push_colored(
                tile,
                &Transform::at(at + Vec3::Y * 0.01).with_scale(vec3(
                    RING_WIDE,
                    0.02,
                    RING_WIDE * 3.0,
                )),
                LINE,
            );
        }

        for marble in 0..MARBLES {
            let body = self.run.bodies[marble + 1];
            let color = if self.run.is_out(marble) {
                GONE
            } else {
                MARBLE
            };

            scene.push_colored(
                sphere,
                &Transform::at(body.position).with_scale(Vec3::splat(MARBLE_RADIUS * 2.0)),
                color,
            );
        }

        let shooter = self.run.shooter();
        scene.push_colored(
            sphere,
            &Transform::at(shooter).with_scale(Vec3::splat(MARBLE_RADIUS * 2.0)),
            SHOOTER_COLOR,
        );

        // the shot, drawn as beads along it, as long as it is hard
        if self.run.phase() == Phase::Aiming {
            let along = self.way();
            let far = 1.0 + self.power / HARDEST * 5.0;

            for bead in 1..=8 {
                let out = bead as f32 / 8.0 * far;
                if out > far {
                    break;
                }

                scene.push_colored(
                    sphere,
                    &Transform::at(shooter + along * (MARBLE_RADIUS + out))
                        .with_scale(Vec3::splat(0.12)),
                    PULL,
                );
            }
        }
    }

    fn process_keyboard(&mut self, input: KeyboardInput) {
        let down = input.state == KeyboardKeyState::Pressed;

        match input.key {
            KeyboardKey::Left => self.turning[0] = down,
            KeyboardKey::Right => self.turning[1] = down,
            KeyboardKey::R if down => {
                if self.run.phase() == Phase::Over {
                    self.run = Run::new();
                    self.power = 0.0;
                }
            }
            KeyboardKey::Escape if down => self.quitting = true,
            _ => {}
        }
    }

    fn process_mouse(&mut self, input: MouseInput) {
        if input.button != MouseButton::Left {
            return;
        }

        if input.is_pressed() {
            if self.run.phase() == Phase::Aiming {
                self.charging = true;
                self.power = shot::SOFTEST;
            }
            return;
        }

        if self.charging {
            self.charging = false;
            self.run.shoot(self.way(), self.power);
            self.power = 0.0;
        }
    }

    /// The cursor moved, so the shot points at wherever it is on the table.
    ///
    /// Absolute, not a turn by however far the mouse went. Raw mouse motion
    /// keeps arriving when the cursor is off the window, so the shot went on
    /// turning while you were somewhere else, and nothing you could do with it
    /// was pointing at a marble: only turning towards one and overshooting.
    /// This only fires while the cursor is over the window, which is the whole
    /// of that first problem.
    fn cursor_moved(&mut self, position: Vec2) {
        self.pointing = Some(position);
    }

    fn is_quitting(&self) -> bool {
        self.quitting
    }

    fn focus_changed(&mut self, _focus: bool) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ring;

    #[test]
    fn the_shot_points_where_it_is_aimed() {
        let mut game = CaromGame::new();

        // the shooter starts on the far z edge, so the opening aim crosses the
        // ring rather than leaving it
        let toward_the_middle = shot::aim(ring::shoot_from(), Vec3::ZERO);
        assert!(
            game.way().dot(toward_the_middle) > 0.99,
            "it opens pointing {:?}",
            game.way()
        );

        game.aim += std::f32::consts::FRAC_PI_2;
        assert!(game.way().dot(toward_the_middle).abs() < 1e-5);
    }

    /// Runs a frame's draw, which is where the aim is taken from the cursor.
    fn aimed(game: &mut CaromGame, cursor: Vec2) -> Vec3 {
        let mut scene = Scene::new();
        let mut camera = Camera::new();

        game.cursor_moved(cursor);
        game.draw(&mut scene, &mut camera);

        game.way()
    }

    #[test]
    fn the_cursor_points_the_shot() {
        // absolute, not a turn by however far the mouse went. The default
        // camera's viewport is one by one, so the middle of it is a half.
        let mut game = CaromGame::new();

        let middle = aimed(&mut game, vec2(0.5, 0.5));
        let toward = shot::aim(game.run.shooter(), Vec3::ZERO);

        assert!(
            middle.dot(toward) > 0.99,
            "the middle of the window aims {:?}, not {:?}",
            middle,
            toward
        );

        let left = aimed(&mut game, vec2(0.2, 0.5));
        let right = aimed(&mut game, vec2(0.8, 0.5));

        assert!(left.x < middle.x, "left of the middle aimed right");
        assert!(right.x > middle.x, "right of the middle aimed left");
        assert_eq!(left.y, 0.0, "the shot left the table");
    }

    #[test]
    fn a_cursor_off_the_window_does_not_move_the_shot() {
        // it did: raw mouse motion keeps arriving when the cursor is somewhere
        // else entirely, so the shot went on turning while you were not even
        // looking at the game. cursor_moved only fires over the window, so
        // the shot simply stays where it was pointed.
        let mut game = CaromGame::new();
        let was = aimed(&mut game, vec2(0.3, 0.5));

        let mut scene = Scene::new();
        let mut camera = Camera::new();
        game.draw(&mut scene, &mut camera);

        assert_eq!(game.way(), was, "the shot drifted with nothing pointing it");
    }

    #[test]
    fn a_cursor_above_the_horizon_points_at_nothing() {
        // a ray that runs away from the table never meets it, and the shot
        // keeps whatever it had rather than taking a direction from nowhere
        let mut game = CaromGame::new();
        let was = aimed(&mut game, vec2(0.5, 0.5));

        let after = aimed(&mut game, vec2(0.5, -4.0));

        assert_eq!(after, was);
    }

    #[test]
    fn a_click_with_no_hold_still_shoots() {
        // it sent nothing, rolled nowhere and gave the turn straight back,
        // which reads as the game ignoring you
        let mut game = CaromGame::new();

        game.process_mouse(MouseInput::new(
            MouseButton::Left,
            blitzkit::mouse::ButtonState::Pressed,
        ));
        game.process_mouse(MouseInput::new(
            MouseButton::Left,
            blitzkit::mouse::ButtonState::Released,
        ));

        assert_eq!(game.run.shots(), 1, "the click went nowhere");
        assert_eq!(game.run.phase(), Phase::Rolling);
        assert!(
            game.run.bodies[0].velocity.length() >= shot::SOFTEST - 1e-5,
            "it left at {}",
            game.run.bodies[0].velocity.length()
        );
    }

    #[test]
    fn holding_the_button_builds_the_shot() {
        let mut game = CaromGame::new();
        game.process_mouse(MouseInput::new(
            MouseButton::Left,
            blitzkit::mouse::ButtonState::Pressed,
        ));
        let softest = game.power;

        let mut text = TextRenderer::new();
        let sound = SoundSystem::new();
        let mut geometry = Geometry::new();
        for _ in 0..30 {
            game.update(1.0 / 60.0, &mut geometry, &mut text, &sound);
        }

        assert!(game.power > softest, "nothing built up");
        assert!(game.power <= HARDEST, "it went past the hardest shot");
    }

    #[test]
    fn letting_go_takes_the_shot() {
        let mut game = CaromGame::new();
        game.charging = true;
        game.power = HARDEST;

        game.process_mouse(MouseInput::new(
            MouseButton::Left,
            blitzkit::mouse::ButtonState::Released,
        ));

        assert_eq!(game.run.phase(), Phase::Rolling);
        assert_eq!(game.run.shots(), 1);
        assert_eq!(game.power, 0.0, "the pull stayed wound up");
    }

    #[test]
    fn the_readout_does_not_pile_up() {
        // it did, by a line a frame, until the text buffer outgrew the device
        let mut game = CaromGame::new();
        let mut text = TextRenderer::new();
        let sound = SoundSystem::new();
        let mut geometry = Geometry::new();

        game.update(1.0 / 60.0, &mut geometry, &mut text, &sound);
        let after_one = text.render_texts.len();

        for _ in 0..100 {
            game.update(1.0 / 60.0, &mut geometry, &mut text, &sound);
        }

        assert_eq!(text.render_texts.len(), after_one, "the readout piled up");
        assert!(after_one > 0, "nothing was drawn at all");
    }

    #[test]
    fn r_racks_them_again() {
        let mut game = CaromGame::new();

        // clear the ring the short way, then let the step notice
        game.run.shoot(game.way(), 1.0);
        for marble in 0..MARBLES {
            game.run.bodies[marble + 1].position = vec3(ring::RING_RADIUS * 3.0, 0.5, 0.0);
            game.run.bodies[marble + 1].velocity = Vec3::ZERO;
        }
        game.run.bodies[0].velocity = Vec3::ZERO;
        game.run.step(shot::STEP);

        assert_eq!(game.run.phase(), Phase::Over, "the ring is not empty");

        game.process_keyboard(KeyboardInput::new(
            KeyboardKey::R,
            KeyboardKeyState::Pressed,
            false,
        ));

        assert_eq!(game.run.phase(), Phase::Aiming, "r did not rack them again");
        assert_eq!(game.run.left(), MARBLES);
        assert_eq!(game.run.shots(), 0);
    }

    #[test]
    fn one_shot_is_not_one_shots() {
        assert_eq!(shots(0), "0 shots");
        assert_eq!(shots(1), "1 shot");
        assert_eq!(shots(2), "2 shots");
    }

    #[test]
    fn the_readout_lines_are_evenly_spaced() {
        let game = CaromGame::new();
        let lines: Vec<Vec2> = (0..game.readout().len()).map(|n| game.line(n)).collect();

        for pair in lines.windows(2) {
            assert_eq!(pair[0].x, pair[1].x);
            assert_eq!(pair[1].y - pair[0].y, HUD_APART);
        }

        assert!(
            game.line(1).y - game.line(0).y > HUD_SIZE,
            "the lines would touch"
        );
    }
}
