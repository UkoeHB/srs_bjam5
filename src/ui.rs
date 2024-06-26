use bevy::prelude::*;
//use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;

//use bevy_cobweb_ui::sickle::prelude::*;
use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn spawn_game_over_screen(mut _c: Commands, mut _s: ResMut<SceneLoader>) {}

//-------------------------------------------------------------------------------------------------------------------

fn setup_ui(mut _c: Commands, mut _s: ResMut<SceneLoader>)
{
    /*
    let scene = LoadableRef::new("ui", "in_game");

    c.ui_builder(UiRoot).load_scene(&mut s, scene, |l| {
        l.disable_picking();
    });
    */
}

//-------------------------------------------------------------------------------------------------------------------

pub struct UiPlugin;

impl Plugin for UiPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_systems(OnExit(GameState::Loading), setup_ui)
            .add_systems(OnEnter(GameState::GameOver), spawn_game_over_screen);
    }
}

//-------------------------------------------------------------------------------------------------------------------
