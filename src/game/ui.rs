use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use bevy_cobweb_ui::sickle::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn spawn_game_hud(mut c: Commands, mut s: ResMut<SceneLoader>, constants: ReactRes<GameConstants>)
{
    let file = LoadableRef::from_file("ui.game_hud");
    let scene = file.e("scene");
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

        // todo: passive/active ability slots

        fn slot_builder<'a>(l: &mut LoadedScene<'a, '_, UiBuilder<'a, Entity>>, file: &LoadableRef, index: usize, target_ability_type: AbilityType) {
            l.load_scene(file.e("ability_slot_scene"), |l| {
                let level_entity = l.get_entity("level").unwrap();
                let level_text_entity = l.get_entity("level::text").unwrap();

                l.edit("icon", |l| {
                    l.update_on(resource_mutation::<PlayerPowerups>(),
                        |id| move |
                            mut c: Commands,
                            mut e: TextEditor,
                            player_powerups: ReactRes<PlayerPowerups>,
                            powerup_bank: Res<PowerupBank>
                        | {
                            // Find the current powerup corresponding to this slot.
                            let Some((_, (info, level))) = player_powerups
                                .iter()
                                .filter_map(|l| {
                                    let Some(info) = powerup_bank.get(&l.name) else {
                                        tracing::error!("player powerups has powerup {} not known to powerup bank", l.name);
                                        return None;
                                    };
                                    if info.ability_type != target_ability_type {
                                        return None;
                                    }
                                    Some((info, l))
                                })
                                .enumerate()
                                .find(|(i, _)| *i == index)
                            else {
                                return;
                            };

                            // Update level text.
                            c.entity(level_entity).insert_reactive(DisplayControl::Display);
                            write_text!(e, level_text_entity, "{}", level.level);

                            // Update icon.
                            c.entity(id).insert_derived(LoadedUiImage{ texture: info.icon.clone(), ..default() });
                        }
                    );
                });
            });
        }

        l.edit("footer::passives::slots", |l| {
            for i in 0..constants.num_passive_slots {
                slot_builder(l, &file, i, AbilityType::Passive);
            }
        });

        l.edit("footer::actives::slots", |l| {
            for i in 0..constants.num_passive_slots {
                slot_builder(l, &file, i, AbilityType::Active);
            }
        });
    });
}

//-------------------------------------------------------------------------------------------------------------------

fn spawn_power_up_ui(
    mut c: Commands,
    mut time: ResMut<Time<Virtual>>,
    mut rng: ResMut<GameRng>,
    constants: ReactRes<GameConstants>,
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
    let options = get_powerup_options(&constants, &mut rng, powerup_source, &player_powerups, &powerup_bank);
    debug_assert!(options.len() > 0);
    let is_filler = options
        .iter()
        .any(|o| matches!(o, PowerupOption::Filler(..)));

    let file = LoadableRef::from_file("ui.power_up");
    let scene = file.e("scene");
    c.ui_builder(UiRoot).load_scene(&mut s, scene, |l| {
        let scene_id = l.id();

        for option in options {
            l.load_scene(file.e("powerup_frame_scene"), |l| {
                // Add custom behavior and styling for the specific power-up.
                let button_id = l.id();
                match option {
                    PowerupOption::Powerup(powerup_type) => {
                        l.load_scene(file.e("powerup_scene"), |l| {
                            let (info, effect_text) = match &powerup_type {
                                PowerupType::New(name) => (
                                    powerup_bank.get(name).cloned().unwrap_or_default(),
                                    String::from("New!"),
                                ),
                                PowerupType::Upgrade(name) => {
                                    let level = player_powerups.get(name);
                                    let next_level = level + 1;
                                    (
                                        powerup_bank.get(name).cloned().unwrap_or_default(),
                                        format!("Lv. {} -> {}", level, next_level),
                                    )
                                }
                            };
                            l.edit("icon", |l| {
                                l.insert_derived(LoadedUiImage { texture: info.icon.clone(), ..default() });
                            });
                            l.edit("title", |l| {
                                let name = info.name.clone();
                                l.update_on((), |id| {
                                    move |mut e: TextEditor| {
                                        write_text!(e, id, "{}: {}", name, effect_text);
                                    }
                                });
                            });
                            l.edit("description", |l| {
                                let description = info.description.clone();
                                l.update_on((), |id| {
                                    move |mut e: TextEditor| {
                                        write_text!(e, id, "{}", description);
                                    }
                                });
                            });

                            l.commands()
                                .ui_builder(button_id)
                                .on_pressed(move |w: &mut World| {
                                    w.syscall(powerup_type.clone(), PowerupType::apply);
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

                #[cfg(feature = "dev")]
                {
                    l.update_on(broadcast::<CancelPowerup>(), |_| {
                        move |event: BroadcastEvent<CancelPowerup>,
                              mut c: Commands,
                              mut time: ResMut<Time<Virtual>>| {
                            let Some(_) = event.try_read() else { return };
                            c.entity(scene_id).despawn_recursive();
                            time.unpause();
                        }
                    });
                }
            });
        }

        if is_filler {
            l.load_scene(file.e("filler_notification"), |_| {});
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
        app.register_type::<DisplayControl>()
            .react(|rc| rc.on_persistent(broadcast::<GamePlay>(), spawn_game_hud))
            .react(|rc| rc.on_persistent(broadcast::<PlayerPowerUp>(), spawn_power_up_ui))
            .react(|rc| rc.on_persistent(broadcast::<PlayerDied>(), spawn_day_failed_ui))
            .react(|rc| rc.on_persistent(broadcast::<PlayerSurvived>(), spawn_day_survived_ui));
    }
}

//-------------------------------------------------------------------------------------------------------------------
