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
        app.add_plugins(Lighting2dPlugin::default());
        app.add_systems(OnEnter(GameState::Play), setup_light);
        app.add_systems(Update, update_light.run_if(in_state(GameState::Play)));
    }
}

fn setup_light(mut c: Commands)
{
    c.spawn((PointLight2dBundle {
        point_light: PointLight2d {
            intensity: 0.0,
            radius: 0.0,
            falloff: 0.0,
            color: Color::WHITE,
        },
        ..default()
    },));
}

fn update_light(mut light: ResMut<AmbientLight2d>, clock: Res<GameClock>, constants: ReactRes<GameConstants>)
{
    let day_progress = clock.elapsed_secs() as f32 / constants.day_length_secs as f32;

    // get brighter until halfway through the day, then start going down
    // parabolas are fun sometimes i guess. desmos graph i used for tweaking: https://www.desmos.com/calculator/jxilotpz1s
    light.brightness = (-2.3 * day_progress * day_progress) + (2. * day_progress) + 0.5;
    println!("{}, {}", day_progress, light.brightness);
}
