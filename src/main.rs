#![allow(clippy::too_many_arguments)]

mod day_start;
mod events;
mod game;
mod game_constants;
mod plugin;
mod utils;

pub use day_start::*;
pub use events::*;
pub use game::*;
pub use game_constants::*;
pub use plugin::*;
pub use utils::*;

fn main()
{
    bevy::app::App::new().add_plugins(AppPlugin).run();
}
