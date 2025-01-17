use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bevy::prelude::*;
use bevy::render::view::screenshot::ScreenshotManager;
use bevy::window::PrimaryWindow;
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
    skip_power_up: KeyCode,
    add_exp: KeyCode,
    apply_damage: KeyCode,
    screenshot: KeyCode,
    show_hide_dev_controls: KeyCode,
}

impl DevControls
{
    fn display(&self) -> String
    {
        // How to get this from the fields of self? Kind of a pain..
        format!("DEV:\nSurvive(Z)\nDie(X)\n+Karma(F)\n+PowerUp(Q)\n\
            SkipPowerup(R)\n+Exp(E)\n-Hp(C)\nScreenshot(P)\nShowHideDev(H)")
    }
}

impl Default for DevControls
{
    fn default() -> Self
    {
        Self {
            survive_immediately: KeyCode::KeyZ,
            die_immediately: KeyCode::KeyX,
            add_karma: KeyCode::KeyF,
            get_power_up: KeyCode::KeyQ,
            skip_power_up: KeyCode::KeyR,
            add_exp: KeyCode::KeyE,
            apply_damage: KeyCode::KeyC,
            screenshot: KeyCode::KeyP,
            show_hide_dev_controls: KeyCode::KeyH,
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn display_dev_controls(mut c: Commands, controls: Res<DevControls>)
{
    c.ui_builder(UiRoot).container(NodeBundle::default(), |ui| {
        ui.despawn_on_broadcast::<GameDayStart>();
        ui.update_on(broadcast::<ToggleDevControls>(), |id| {
            move |mut off: Local<bool>, event: BroadcastEvent<ToggleDevControls>, mut c: Commands| {
                if event.try_read().is_none() {
                    return;
                };
                let next = match *off {
                    false => DisplayControl::Hide,
                    true => DisplayControl::Display,
                };
                c.entity(id).insert_reactive(next);
                *off = !*off;
            }
        });

        ui.style()
            .height(Val::Vh(100.0))
            .flex_direction(FlexDirection::Column)
            .justify_content(JustifyContent::Center);

        ui.container(NodeBundle::default(), |ui| {
            ui.insert_derived(TextLine::from_text(controls.display()));
        });
    });
}

//-------------------------------------------------------------------------------------------------------------------

fn check_dev_commands(
    mut last_command: Local<Duration>,
    time: Res<Time<Real>>,
    mut c: Commands,
    button_input: Res<ButtonInput<KeyCode>>,
    controls: Res<DevControls>,
    mut karma: ReactResMut<Karma>,
    mut powerups: ResMut<BufferedPowerUps>,
    mut player: Query<(Entity, &mut Level, &Health)>,
    mut damage: EventWriter<DamageEvent>,
    main_window: Query<Entity, With<PrimaryWindow>>,
    mut screenshot: ResMut<ScreenshotManager>,
)
{
    if *last_command + Duration::from_millis(150) > time.elapsed() {
        return;
    }

    for pressed in button_input.get_pressed() {
        if *pressed == controls.survive_immediately {
            c.react().broadcast(PlayerSurvived);
        } else if *pressed == controls.die_immediately {
            c.react().broadcast(PlayerDied);
        } else if *pressed == controls.add_karma {
            karma.get_mut(&mut c).add(25);
        } else if *pressed == controls.get_power_up {
            if powerups.is_handling_powerup() {
                continue;
            }
            powerups.insert([PowerupSource::LevelUp]);
        } else if *pressed == controls.skip_power_up {
            if !powerups.is_handling_powerup() {
                continue;
            }
            powerups.end_handling_powerup();
            c.react().broadcast(CancelPowerup);
        } else if *pressed == controls.add_exp {
            let Ok((_, mut level, _)) = player.get_single_mut() else { continue };
            let required = level.exp_required() as usize;
            let levels = level.add_exp(required / 3 + required / 7 + 1, &ExpAmp::new(0));
            powerups.insert(levels.iter().map(|_| PowerupSource::LevelUp));
        } else if *pressed == controls.apply_damage {
            let Ok((entity, _, health)) = player.get_single_mut() else { continue };
            let max = health.max();
            damage.send(DamageEvent {
                source: Entity::PLACEHOLDER,
                target: entity,
                damage: max / 5 + max / 7 + 1,
            });
        } else if *pressed == controls.screenshot {
            let time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default();
            let secs = time.as_secs() % 60;
            let mins = (time.as_secs() / 60) % 60;
            let hrs = ((time.as_secs() / 60) / 60) % 24;
            let days = (((time.as_secs() / 60) / 60) / 24) % 365;
            let year = (((time.as_secs() / 60) / 60) / 24) / 365 + 1970;
            let path = format!("./screenshot-{}:{}:{:0>2}:{:0>2}:{:0>2}.png", year, days, hrs, mins, secs);
            screenshot
                .save_screenshot_to_disk(main_window.single(), path)
                .unwrap();
        } else if *pressed == controls.show_hide_dev_controls {
            c.react().broadcast(ToggleDevControls);
        } else {
            continue;
        }

        *last_command = time.elapsed();
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Reactive event for canceling powerups.
pub struct CancelPowerup;

/// Reactive event for toggling the display of dev controls.
pub struct ToggleDevControls;

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
