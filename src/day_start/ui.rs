use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use bevy_cobweb_ui::sickle::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn spawn_day_start_ui(mut c: Commands, mut s: ResMut<SceneLoader>)
{
    let scene = LoadableRef::new("ui.day_start", "scene");
    c.ui_builder(UiRoot).load_scene(&mut s, scene, |l| {
        l.despawn_on_broadcast::<GamePlay>();

        l.edit("header::day::text", |l| {
            l.update_on(resource_mutation::<Day>(), |id| {
                move |mut e: TextEditor, day: ReactRes<Day>, spawnsched: Res<SpawnSchedule>| {
                    // Hacky end condition.
                    if day.get() > spawnsched.num_scheduled() {
                        write_text!(e, id, "You reached the end of this jam game!\nThe last day will repeat.\nDay {}", day.get());
                    } else {
                        write_text!(e, id, "Day {}", day.get());
                    }
                }
            });
        });

        l.edit("header::karma::text", |l| {
            l.update_on(resource_mutation::<Karma>(), |id| {
                move |mut e: TextEditor, karma: ReactRes<Karma>| {
                    write_text!(e, id, "Karma {}", karma.total());
                }
            });
        });

        // todo: display upgrades (as info cards/buttons in a scroll-view)

        l.edit("footer::start_button", |l| {
            l.on_pressed(|mut c: Commands| {
                c.react().broadcast(GamePlay);
            });
        });
    });
}

//-------------------------------------------------------------------------------------------------------------------

pub struct StartUiPlugin;

impl Plugin for StartUiPlugin
{
    fn build(&self, app: &mut App)
    {
        app.react(|rc| rc.on_persistent(broadcast::<GameDayStart>(), spawn_day_start_ui));
    }
}

//-------------------------------------------------------------------------------------------------------------------
