use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_lit::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn update_light(mut light: ResMut<AmbientLight2d>, clock: Res<GameClock>, constants: ReactRes<GameConstants>)
{
    let day_progress = clock.elapsed.as_secs_f32() / constants.day_length_secs as f32;

    // get brighter until halfway through the day, then start going down
    // parabolas are fun sometimes i guess. desmos graph i used for tweaking: https://www.desmos.com/calculator/jxilotpz1s
    light.brightness = (-2.3 * day_progress * day_progress) + (2. * day_progress) + 0.5;
}

//-------------------------------------------------------------------------------------------------------------------

pub struct LightPlugin;

impl Plugin for LightPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(Lighting2dPlugin::default())
            .add_systems(Update, update_light.run_if(in_state(PlayState::Day)));
    }
}

//-------------------------------------------------------------------------------------------------------------------
