use std::fmt::Debug;

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy::state::state::FreelyMutableState;
use bevy::window::WindowTheme;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use bevy_cobweb_ui::sickle::SickleUiPlugin;
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_spritesheet_animation::plugin::SpritesheetAnimationPlugin;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn set_state<T: FreelyMutableState + Debug>(state: T) -> impl FnMut(ResMut<NextState<T>>)
{
    move |mut next: ResMut<NextState<T>>| {
        tracing::info!("entering state {:?}", state);
        next.set(state.clone());
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn handle_loading_done(mut c: Commands)
{
    c.react().broadcast(GameDayStart);
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(States, Default, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum GameState
{
    #[default]
    Loading,
    DayStart,
    Play,
    DayOver,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component)]
pub struct MainCamera;

fn setup_camera(mut c: Commands)
{
    c.spawn((
        Camera2dBundle {
            projection: OrthographicProjection { far: 1000., near: -1000., scale: 0.5, ..default() },
            ..default()
        },
        MainCamera,
    ));
}

//-------------------------------------------------------------------------------------------------------------------

pub struct AppPlugin;

impl Plugin for AppPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Surviving Today".to_string(),
                        window_theme: Some(WindowTheme::Dark),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .set(AssetPlugin { meta_check: AssetMetaCheck::Never, ..default() }),
        )
        // Dependencies
        .add_plugins(DefaultPickingPlugins) //todo: need this?
        .add_plugins(SpritesheetAnimationPlugin)
        .add_plugins(ReactPlugin)
        .add_plugins(SickleUiPlugin)
        .add_plugins(CobwebUiPlugin)
        // Utils
        .add_plugins(UtilsPlugin) // must be added after CobwebUiPlugin
        // Game content
        .add_plugins(DayStartPlugin)
        .add_plugins(GamePlugin)
        .add_plugins(GameConstantsPlugin)
        // Load all assets
        .load("manifest.caf.json")
        // Misc setup and game management
        .init_state::<GameState>()
        .enable_state_scoped_entities::<GameState>()
        .add_systems(Startup, setup_camera)
        .add_systems(OnEnter(LoadState::Done), handle_loading_done)
        .react(|rc| rc.on_persistent(broadcast::<GameDayStart>(), set_state(GameState::DayStart)))
        .react(|rc| rc.on_persistent(broadcast::<GamePlay>(), set_state(GameState::Play)))
        .react(|rc| rc.on_persistent(broadcast::<GameDayOver>(), set_state(GameState::DayOver)));
    }
}

//-------------------------------------------------------------------------------------------------------------------
