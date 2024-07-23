use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use bevy_cobweb_ui::sickle::prelude::*;

use crate::*;

// button to add exp
// button to take dmg

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Debug, Clone)]
struct DevControls
{
    survive_immediately: KeyCode,
    die_immediately: KeyCode,
    add_karma: KeyCode,
    get_power_up: KeyCode,
}

impl DevControls
{
    fn display(&self) -> String
    {
        // How to get this from the fields of self? Kind of a pain..
        format!("DEV: Survive(S), Die(D), +Karma(K), +PowerUp(P)")
    }
}

impl Default for DevControls
{
    fn default() -> Self
    {
        Self {
            survive_immediately: KeyCode::KeyS,
            die_immediately: KeyCode::KeyD,
            add_karma: KeyCode::KeyK,
            get_power_up: KeyCode::KeyP,
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn display_dev_controls(mut c: Commands, controls: Res<DevControls>)
{
    tracing::error!("dev controls");

    c.ui_builder(UiRoot).container(NodeBundle::default(), |ui| {
        ui.despawn_on_broadcast::<GameDayStart>();

        ui.style()
            .height(Val::Vh(100.0))
            .flex_direction(FlexDirection::Column)
            .justify_content(JustifyContent::FlexEnd);

        ui.container(NodeBundle::default(), |ui| {
            ui.insert_derived(TextLine { text: controls.display(), ..default() });
        });
    });
}

//-------------------------------------------------------------------------------------------------------------------

fn check_dev_commands(
    mut c: Commands,
    button_input: Res<ButtonInput<KeyCode>>,
    controls: Res<DevControls>,
    mut karma: ReactResMut<Karma>,
    mut powerups: ResMut<BufferedPowerUps>,
)
{
    for pressed in button_input.get_pressed() {
        if *pressed == controls.survive_immediately {
            c.react().broadcast(PlayerSurvived);
        } else if *pressed == controls.die_immediately {
            c.react().broadcast(PlayerDied);
        } else if *pressed == controls.add_karma {
            karma.get_mut(&mut c).add(10);
        } else if *pressed == controls.get_power_up {
            if powerups.is_handling_powerup() {
                continue;
            }
            powerups.insert([PowerUpSource::LevelUp]);
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct DevPlugin;

impl Plugin for DevPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_resource::<DevControls>()
            .add_systems(OnEnter(GameState::Play), display_dev_controls)
            .add_systems(PreUpdate, check_dev_commands.run_if(in_state(PlayState::Day)));
    }
}

//-------------------------------------------------------------------------------------------------------------------
