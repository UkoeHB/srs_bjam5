use ::bevy_lit::prelude::*;
use bevy::color::palettes::css::*;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;

use crate::*;

pub struct LightPlugin;

impl Plugin for LightPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(Lighting2dPlugin {
            ambient_light: AmbientLight2d { brightness: 0.9, color: Color::Srgba(GREEN) },
            shadow_softness: 32.0,
        });
        app.add_systems(Update, update_light.run_if(in_state(GameState::Play)));
    }
}

// fn setup_light(mut c: Commands)

fn update_light(mut light: ResMut<AmbientLight2d>, clock: Res<GameClock>, constants: ReactRes<GameConstants>)
{
    let brightness = clock.elapsed_secs() as f32 / constants.day_length_secs as f32;
    light.brightness = brightness.clamp(0.1, 10.);
    println!("{:?}", light.brightness);
}
