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

// todo: freeze time on PlayerLevelUp, then unfreeze when option selected
fn spawn_power_up_ui(mut c: Commands, mut s: ResMut<SceneLoader>)
{
    // todo: select power-up options

    let scene = LoadableRef::new("ui.power_up", "scene");
    c.ui_builder(UiRoot).load_scene(&mut s, scene, |l| {
        let _scene_id = l.id();
        // todo: despawn scene when an option is selected

        // todo: display power-up options (each is a button)
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
            .react(|rc| rc.on_persistent(broadcast::<PlayerLevelUp>(), spawn_power_up_ui))
            .react(|rc| rc.on_persistent(broadcast::<PlayerDied>(), spawn_day_failed_ui))
            .react(|rc| rc.on_persistent(broadcast::<PlayerSurvived>(), spawn_day_survived_ui));
    }
}

//-------------------------------------------------------------------------------------------------------------------
