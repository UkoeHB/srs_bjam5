use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use bevy_cobweb_ui::sickle::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn spawn_day_start_ui(mut c: Commands, mut s: ResMut<SceneLoader>)
{
    let scene = LoadableRef::new("ui.day_start", "scene");
    c.ui_builder(UiRoot).load_scene(&mut s, scene, |l| {
        l.despawn_on_broadcast::<GamePlay>();

        // todo: display current day

        // todo: display current Karma

        // todo: display settings button

        // todo: display upgrades (as info cards/buttons in a scroll-view)

        l.edit("start_button", |l| {
            l.on_pressed(|mut c: Commands| {
                c.react().broadcast(GamePlay);
            });
        });
    });
}

//-------------------------------------------------------------------------------------------------------------------

pub struct StartUiPlugin;

impl Plugin for StartUiPlugin
{
    fn build(&self, app: &mut App)
    {
        app.react(|rc| rc.on_persistent(broadcast::<GameDayStart>(), spawn_day_start_ui));
    }
}

//-------------------------------------------------------------------------------------------------------------------
