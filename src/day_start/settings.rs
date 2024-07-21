use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use bevy_cobweb_ui::sickle::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// We have a separate reactor to do on/off because when the settings menu is triggered (e.g. by pressing Esc),
/// we don't know if it needs to be opened or closed - and the place where we trigger it shouldn't need to figure
/// that out.
fn handle_toggle_settings(mut state: Local<bool>, mut c: Commands)
{
    let prev_state = *state;
    *state = !prev_state;
    match prev_state {
        true => {
            c.react().broadcast(ToggleSettingsOff);
        }
        false => {
            c.react().broadcast(ToggleSettingsOn);
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

//todo: freeze time when toggle-on, unfreeze when toggle-off

fn spawn_settings_menu(mut c: Commands, mut s: ResMut<SceneLoader>)
{
    let scene = LoadableRef::new("ui.settings", "scene");
    c.ui_builder(UiRoot).load_scene(&mut s, scene, |l| {
        l.despawn_on_broadcast::<ToggleSettingsOff>();

        l.edit("window", |l| {
            // todo: controls image (non-configurable)

            // todo: audio slider

            // todo: restart from day 1 button

            l.edit("close_button", |l| {
                l.on_pressed(|mut c: Commands| {
                    c.react().broadcast(ToggleSettingsOff);
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
        app.react(|rc| rc.on_persistent(broadcast::<ToggleSettings>(), handle_toggle_settings))
            .react(|rc| rc.on_persistent(broadcast::<ToggleSettingsOn>(), spawn_settings_menu));
    }
}

//-------------------------------------------------------------------------------------------------------------------
