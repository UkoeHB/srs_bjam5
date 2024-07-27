#![allow(clippy::too_many_arguments)]

mod controls;
mod day_start;
#[cfg(feature = "dev")]
mod dev;
mod events;
mod game;
mod game_constants;
mod meta;
mod plugin;
mod settings;
mod utils;

pub use controls::*;
pub use day_start::*;
#[cfg(feature = "dev")]
pub use dev::*;
pub use events::*;
pub use game::*;
pub use game_constants::*;
pub use meta::*;
pub use plugin::*;
pub use settings::*;
pub use utils::*;

fn main()
{
    bevy::app::App::new().add_plugins(AppPlugin).run();
}
