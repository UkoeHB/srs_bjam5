use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use bevy_cobweb_ui::sickle::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn setup_settings(mut c: Commands, mut s: ResMut<SceneLoader>)
{
    let scene = LoadableRef::new("ui.settings", "button_scene");
    c.ui_builder(UiRoot).load_scene(&mut s, scene, |l| {
        l.edit("button", |l| {
            l.on_pressed(|mut c: Commands| {
                c.react().broadcast(ToggleSettings);
            });
        });
    });
}

//-------------------------------------------------------------------------------------------------------------------

/// We have a separate reactor to do on/off because when the settings menu is triggered (e.g. by pressing Esc),
/// we don't know if it needs to be opened or closed - and the place where we trigger it shouldn't need to figure
/// that out.
fn handle_toggle_settings(
    mut state: Local<bool>,
    mut c: Commands,
    mut time: ResMut<Time<Virtual>>,
    powerup_buffer: Res<BufferedPowerUps>,
)
{
    let prev_state = *state;
    *state = !prev_state;
    match prev_state {
        true => {
            // Unpause time when settings closed, if we aren't in a powerup screen.
            // todo: this is a hacky solution, need a centralized time control system
            if !powerup_buffer.is_handling_powerup() {
                time.unpause();
            }

            c.react().broadcast(ToggleSettingsOff);
        }
        false => {
            // Pause time while in settings.
            time.pause();

            c.react().broadcast(ToggleSettingsOn);
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn spawn_settings_menu(mut c: Commands, mut s: ResMut<SceneLoader>)
{
    let scene = LoadableRef::new("ui.settings", "display_scene");
    c.ui_builder(UiRoot).load_scene(&mut s, scene, |l| {
        l.despawn_on_broadcast::<ToggleSettingsOff>();

        l.edit("window", |l| {
            // todo: controls image (non-configurable)

            // todo: audio slider

            // todo: restart from day 1 button

            l.edit("footer::close_button", |l| {
                l.on_pressed(|mut c: Commands| {
                    c.react().broadcast(ToggleSettings);
                });
            });
        });
    });
}

//-------------------------------------------------------------------------------------------------------------------

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_systems(OnExit(LoadState::Loading), setup_settings)
            .react(|rc| rc.on_persistent(broadcast::<ToggleSettings>(), handle_toggle_settings))
            .react(|rc| rc.on_persistent(broadcast::<ToggleSettingsOn>(), spawn_settings_menu));
    }
}

//-------------------------------------------------------------------------------------------------------------------
