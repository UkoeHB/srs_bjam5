#![allow(clippy::too_many_arguments)]

mod controls;
mod day_start;
mod dev;
mod events;
mod game;
mod game_constants;
mod meta;
mod plugin;
mod utils;

pub use controls::*;
pub use day_start::*;
//pub use dev::*;
pub use events::*;
pub use game::*;
pub use game_constants::*;
pub use meta::*;
pub use plugin::*;
pub use utils::*;

fn main()
{
    bevy::app::App::new().add_plugins(AppPlugin).run();
}
