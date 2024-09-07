use std::fmt::Debug;

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy::window::WindowTheme;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use bevy_cobweb_ui::sickle::SickleUiPlugin;
use bevy_ecs_tilemap::TilemapPlugin;
use bevy_spritesheet_animation::plugin::SpritesheetAnimationPlugin;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn broadcast_event<T: Clone + Send + Sync + 'static>(state: &'static str, event: T) -> impl FnMut(Commands)
{
    move |mut c: Commands| {
        tracing::info!("entered state {}", state);
        c.react().broadcast(event.clone());
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn handle_loading_done(mut c: Commands)
{
    c.set_state(GameState::DayStart);
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(States, Default, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum GameState
{
    #[default]
    Loading,
    DayStart,
    Play,
}

//-------------------------------------------------------------------------------------------------------------------

/// We use substates for Day/DayOver so we can scope entities to GameState::Play. This way they stay alive even
/// after the day result screen pops up.
#[derive(SubStates, Default, Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[source(GameState = GameState::Play)]
pub enum PlayState
{
    #[default]
    Day,
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
                        prevent_default_event_handling: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .set(AssetPlugin { meta_check: AssetMetaCheck::Never, ..default() }),
        )
        // Dependencies
        .add_plugins(TilemapPlugin)
        .add_plugins(SpritesheetAnimationPlugin)
        .add_plugins(ReactPlugin)
        .add_plugins(SickleUiPlugin)
        .add_plugins(CobwebUiPlugin)
        // Utils
        .add_plugins(UtilsPlugin) // must be added after CobwebUiPlugin
        // Game content
        .add_plugins(ControlsPlugin)
        .add_plugins(SettingsPlugin)
        .add_plugins(GameConstantsPlugin)
        .add_plugins(MetaPlugin)
        .add_plugins(DayStartPlugin)
        .add_plugins(GamePlugin)
        // Load all assets
        .load("manifest.caf.json")
        // Misc setup and game management
        .init_state::<GameState>()
        .add_sub_state::<PlayState>()
        .enable_state_scoped_entities::<GameState>()
        .add_systems(Startup, setup_camera)
        .add_systems(OnEnter(LoadState::Done), handle_loading_done)
        .add_systems(
            OnEnter(GameState::DayStart),
            broadcast_event("GameState::DayStart", GameDayStart),
        )
        .add_systems(OnEnter(GameState::Play), broadcast_event("GameState::Play", GamePlay))
        .add_systems(
            OnEnter(PlayState::DayOver),
            broadcast_event("PlayState::DayOver", GameDayOver),
        );

        #[cfg(feature = "dev")]
        {
            app.add_plugins(DevPlugin);
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
