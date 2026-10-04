//! A ring of marbles and one to shoot with. See `specs/`.

mod carom_game;
mod game;
mod ring;
mod shot;

use blitzkit::start;
use carom_game::CaromGame;

/// Whether this run is only here to be photographed, for `refresh-screenshots`
/// in the project above.
pub fn staged() -> bool {
    std::env::args().any(|arg| arg == "--screenshot")
}

fn main() {
    start("carom", Box::new(CaromGame::new()));
}
