use bevy::prelude::*;
use wasm_timer::{SystemTime, UNIX_EPOCH};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

pub struct GamePlugin;

impl Plugin for GamePlugin
{
    fn build(&self, app: &mut App)
    {
        app.insert_resource(GameRng::new(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        ))
        .add_plugins(GameSetupPlugin)
        .add_plugins(SpriteLayersPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------
