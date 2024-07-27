use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn setup_audio(mut c: Commands, constants: ReactRes<GameConstants>, asset_server: Res<AssetServer>)
{
    tracing::info!(constants.loop1);

    c.spawn(AudioBundle {
        source: asset_server.load(&constants.loop1),
        ..default()
    });
}

//-------------------------------------------------------------------------------------------------------------------

pub struct AudioPlugin;

impl Plugin for AudioPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_systems(
            OnExit(LoadState::Loading),
            (setup_audio).chain(),
        );
    }
}

//-------------------------------------------------------------------------------------------------------------------
