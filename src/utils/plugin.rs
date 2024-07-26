use bevy::prelude::*;
use wasm_timer::{SystemTime, UNIX_EPOCH};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

pub struct UtilsPlugin;

impl Plugin for UtilsPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(AssetsPlugin).insert_resource(GameRng::new(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        ));
    }
}

//-------------------------------------------------------------------------------------------------------------------
