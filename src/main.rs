#![allow(clippy::too_many_arguments)]

mod assets;
mod game;
mod game_constants;
mod mod_picking_ext;
mod run_app;
mod ui;

pub use assets::*;
pub use game::*;
pub use game_constants::*;
pub use mod_picking_ext::*;
pub use run_app::*;
pub use ui::*;

fn main()
{
    run_app();
}
