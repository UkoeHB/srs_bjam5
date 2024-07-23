use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use bevy_cobweb_ui::sickle::prelude::*;

use crate::*;

// button to add exp
// button to take dmg
// button to level up
// button to survive automatically
// button to immediately die

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Debug, Clone)]
struct DevControls
{
    add_karma: KeyCode,
}

impl DevControls
{
    fn display(&self) -> String
    {
        // How to get this from the fields of self? Kind of a pain..
        format!("DEV: +Karma(K)")
    }
}

impl Default for DevControls
{
    fn default() -> Self
    {
        Self { add_karma: KeyCode::KeyK }
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
)
{
    for pressed in button_input.get_pressed() {
        if *pressed == controls.add_karma {
            karma.get_mut(&mut c).add(10);
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
