//! A ring of marbles and one to shoot with. See `specs/`.

mod carom_game;
mod game;
mod ring;
mod shot;

use blitzkit::start;
use carom_game::CaromGame;

fn main() {
    start("carom", Box::new(CaromGame::new()));
}
