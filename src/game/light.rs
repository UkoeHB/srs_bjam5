use bevy::color::palettes::basic::*;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_light_2d::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn setup_light(mut c: Commands, camera: Query<Entity, With<MainCamera>>)
{
    c.entity(camera.single())
        .insert(AmbientLight2d { color: Color::Srgba(WHITE), brightness: 1.0 });
}

//-------------------------------------------------------------------------------------------------------------------

fn update_light(mut light: Query<&mut AmbientLight2d>, clock: Res<GameClock>, constants: ReactRes<GameConstants>)
{
    let mut light = light.single_mut();
    let day_progress = clock.elapsed.as_secs_f32() / constants.day_length_secs as f32;

    // get brighter until halfway through the day, then start going down
    // parabolas are fun sometimes i guess. desmos graph i used for tweaking: https://www.desmos.com/calculator/jxilotpz1s
    //light.brightness = (-3. * day_progress * day_progress) + (3. * day_progress) + 0.2;
    //light.brightness = (-2.47 * day_progress * day_progress) + (2.33 * day_progress) + 0.35;
    //light.brightness = (-2.3 * day_progress * day_progress) + (2.0 * day_progress) + 0.5;
    light.brightness = (-1.45 * day_progress * day_progress) + (1.45 * day_progress) + 0.64;
}

//-------------------------------------------------------------------------------------------------------------------

pub struct LightPlugin;

impl Plugin for LightPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(Light2dPlugin)
            .add_systems(PostStartup, setup_light)
            .add_systems(Update, update_light.run_if(in_state(PlayState::Day)));
    }
}

//-------------------------------------------------------------------------------------------------------------------
