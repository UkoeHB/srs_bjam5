use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy::window::WindowTheme;
use bevy_cobweb::react::ReactPlugin;
use bevy_cobweb_ui::prelude::*;
use bevy_cobweb_ui::sickle::SickleUiPlugin;
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_spritesheet_animation::plugin::SpritesheetAnimationPlugin;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn enter_play_state(mut next: ResMut<NextState<GameState>>)
{
    next.set(GameState::Play);
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(States, Default, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum GameState
{
    #[default]
    Loading,
    Play,
    GameOver,
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

pub fn run_app()
{
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "SRS Bevy Jam 5".to_string(),
                        window_theme: Some(WindowTheme::Dark),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .set(AssetPlugin { meta_check: AssetMetaCheck::Never, ..default() }),
        )
        // Dependencies
        .add_plugins(DefaultPickingPlugins)
        .add_plugins(SpritesheetAnimationPlugin)
        .add_plugins(ReactPlugin)
        .add_plugins(SickleUiPlugin)
        .add_plugins(CobwebUiPlugin)
        // Tools (todo: move to bevy_cobweb_ui)
        .add_plugins(AssetsPlugin) // must be added after CobwebUiPlugin
        .add_plugins(ModPickingExtPlugin)
        // Game content
        .add_plugins(UiPlugin)
        .add_plugins(GamePlugin)
        .add_plugins(GameConstantsPlugin)
        // Load all assets
        .load("manifest.caf.json")
        // Misc setup and game management
        .init_state::<GameState>()
        .add_systems(Startup, setup_camera)
        .add_systems(OnEnter(LoadState::Done), enter_play_state)
        .run();
}

//-------------------------------------------------------------------------------------------------------------------
