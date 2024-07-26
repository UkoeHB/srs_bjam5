use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use bevy_cobweb_ui::sickle::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn spawn_game_hud(mut c: Commands, mut s: ResMut<SceneLoader>)
{
    let scene = LoadableRef::new("ui.game_hud", "scene");
    c.ui_builder(UiRoot).load_scene(&mut s, scene, |l| {
        l.despawn_on_broadcast::<GameDayStart>();

        l.edit("header::day::text", |l| {
            l.update_on((), |id| {
                move |mut e: TextEditor, day: ReactRes<Day>| {
                    write_text!(e, id, "Day {}", day.get());
                }
            });
        });

        l.edit("header::clock", |l| {
            l.update_on(broadcast::<GameClockIncremented>(), |id| {
                move |mut e: TextEditor, clock: Res<GameClock>| {
                    let secs = clock.elapsed_secs() % 60;
                    let mins = (clock.elapsed_secs() / 60) % 60;
                    write_text!(e, id, "{:0>1}:{:0>2}", mins, secs);
                }
            });
        });

        l.edit("header::karma::text", |l| {
            l.update_on(resource_mutation::<Karma>(), |id| {
                move |mut e: TextEditor, karma: ReactRes<Karma>| {
                    write_text!(e, id, "Karma {}", karma.day_collected());
                }
            });
        });

        // todo: settings button

        // todo: health and exp bars w/ level number

        // todo: passive/active ability slots
    });
}

//-------------------------------------------------------------------------------------------------------------------

fn spawn_power_up_ui(
    mut c: Commands,
    mut time: ResMut<Time<Virtual>>,
    mut rng: ResMut<GameRng>,
    player_powerups: ReactRes<PlayerPowerups>,
    powerup_bank: Res<PowerupBank>,
    mut s: ResMut<SceneLoader>,
    mut powerups: ResMut<BufferedPowerUps>,
)
{
    let Some(powerup_source) = powerups.current_powerup() else {
        tracing::error!("powerup source missing in spawn_power_up_ui");
        powerups.end_handling_powerup();
        return;
    };

    // Pause time now that we're spawning a power-up sequence.
    time.pause();

    // Generate power-up options for the player.
    let options = get_powerup_options(&mut rng, powerup_source, &player_powerups, &powerup_bank);
    debug_assert!(options.len() > 0);

    let file = LoadableRef::from_file("ui.power_up");
    let scene = file.e("scene");
    c.ui_builder(UiRoot).load_scene(&mut s, scene, |l| {
        let scene_id = l.id();

        for option in options {
            l.load_scene(file.e("powerup_frame_scene"), |l| {
                // Add custom behavior and styling for the specific power-up.
                let button_id = l.id();
                match option {
                    PowerupOption::PowerUp => {
                        l.load_scene(file.e("powerup_scene"), |l| {
                            l.commands().ui_builder(button_id).on_pressed(move || {
                                //todo: apply the power up
                            });
                        });
                    }
                    PowerupOption::Filler(filler_type) => {
                        l.load_scene(file.e("filler_scene"), |l| {
                            l.update_on((), |id| {
                                move |mut e: TextEditor, data: Res<FillerDatabase>| {
                                    let (_, description) = data.get_info(filler_type);
                                    write_text!(e, id, "{}", description.as_str());
                                }
                            });
                            l.commands()
                                .ui_builder(button_id)
                                .on_pressed(move |w: &mut World| {
                                    w.syscall(filler_type, FillerType::apply);
                                });
                        });
                    }
                }

                // Add behavior all buttons need.
                // - Add this *after* setting up other on-pressed reactors so the despawn occurs last.
                l.on_pressed(
                    move |mut c: Commands,
                          mut buffer: ResMut<BufferedPowerUps>,
                          mut time: ResMut<Time<Virtual>>| {
                        c.entity(scene_id).despawn_recursive();
                        buffer.end_handling_powerup();

                        // Unpause time now that the power-up sequence is done.
                        time.unpause();
                    },
                );
            });
        }
    });
}

//-------------------------------------------------------------------------------------------------------------------

fn spawn_day_failed_ui(mut c: Commands, mut s: ResMut<SceneLoader>)
{
    let scene = LoadableRef::new("ui.day_result", "failure_scene");
    c.ui_builder(UiRoot).load_scene(&mut s, scene, |l| {
        l.despawn_on_broadcast::<GameDayStart>();

        l.edit("window::today_again_button", |l| {
            l.on_pressed(|mut c: Commands| {
                c.react().broadcast(GameDayStart);
            });
        });
    });
}

//-------------------------------------------------------------------------------------------------------------------

fn spawn_day_survived_ui(mut c: Commands, mut s: ResMut<SceneLoader>)
{
    let scene = LoadableRef::new("ui.day_result", "success_scene");
    c.ui_builder(UiRoot).load_scene(&mut s, scene, |l| {
        l.despawn_on_broadcast::<GameDayStart>();

        l.edit("window::tomorrow_button", |l| {
            l.on_pressed(|mut c: Commands, mut day: ReactResMut<Day>| {
                day.get_mut(&mut c).increment();
                c.react().broadcast(GameDayStart);
            });
        });

        l.edit("window::today_again_button", |l| {
            l.on_pressed(|mut c: Commands| {
                c.react().broadcast(GameDayStart);
            });
        });
    });
}

//-------------------------------------------------------------------------------------------------------------------

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin
{
    fn build(&self, app: &mut App)
    {
        app.react(|rc| rc.on_persistent(broadcast::<GamePlay>(), spawn_game_hud))
            .react(|rc| rc.on_persistent(broadcast::<PlayerPowerUp>(), spawn_power_up_ui))
            .react(|rc| rc.on_persistent(broadcast::<PlayerDied>(), spawn_day_failed_ui))
            .react(|rc| rc.on_persistent(broadcast::<PlayerSurvived>(), spawn_day_survived_ui));
    }
}

//-------------------------------------------------------------------------------------------------------------------
