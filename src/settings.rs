use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use bevy_cobweb_ui::sickle::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

struct SliderChanged;

fn detect_silder_change(mut c: Commands, query: Query<Entity, Changed<Slider>>)
{
    for slider in query.iter() {
        c.react().entity_event(slider, SliderChanged);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Style override for the `sickle_ui` `Slider` widget.
fn adjusted_slider_style(style_builder: &mut StyleBuilder, slider: &Slider, theme_data: &ThemeData)
{
    // This is styling for a horizontal slider.
    {
        style_builder
            .justify_content(JustifyContent::SpaceBetween)
            .align_items(AlignItems::Center)
            .width(Val::Percent(100.))
            .height(Val::Px(4.0))
            .padding(UiRect::horizontal(Val::Px(4.0)));

        style_builder
            .switch_target(Slider::LABEL)
            .margin(UiRect::right(Val::Px(0.0)));

        style_builder
            .switch_target(Slider::BAR_CONTAINER)
            .width(Val::Percent(100.));

        style_builder
            .switch_target(Slider::BAR)
            .width(Val::Percent(100.))
            .height(Val::Px(10.0))
            .margin(UiRect::vertical(Val::Px(4.0)));

        style_builder
            .switch_target(Slider::READOUT)
            .min_width(Val::Px(50.0))
            .margin(UiRect::left(Val::Px(5.0)));

        style_builder
            .switch_context(Slider::HANDLE, None)
            .margin(UiRect::px(-2.0, 0., -10.0, 0.));
    }

    style_builder.reset_context();

    style_builder
        .switch_target(Slider::LABEL)
        .sized_font(SizedFont {
            font: "embedded://sickle_ui/fonts/FiraSans-Regular.ttf".into(),
            size: 25.0,
        })
        .font_color(Color::WHITE);

    if slider.config().label.is_none() {
        style_builder
            .switch_target(Slider::LABEL)
            .display(Display::None)
            .visibility(Visibility::Hidden);
    } else {
        style_builder
            .switch_target(Slider::LABEL)
            .display(Display::Flex)
            .visibility(Visibility::Inherited);
    }

    if !slider.config().show_current {
        style_builder
            .switch_target(Slider::READOUT_CONTAINER)
            .display(Display::None)
            .visibility(Visibility::Hidden);
    } else {
        style_builder
            .switch_target(Slider::READOUT_CONTAINER)
            .display(Display::Flex)
            .visibility(Visibility::Inherited);
    }

    style_builder
        .switch_target(Slider::READOUT)
        .sized_font(SizedFont {
            font: "embedded://sickle_ui/fonts/FiraSans-Regular.ttf".into(),
            size: 25.0,
        })
        .font_color(Color::WHITE);

    style_builder
        .switch_target(Slider::BAR)
        .border(UiRect::px(2., 2.0, 2., 2.0))
        .background_color(Color::Hsla(Hsla {
            hue: 34.0,
            saturation: 0.63,
            lightness: 0.55,
            alpha: 1.0,
        }))
        .border_color(Color::Hsla(Hsla {
            hue: 34.0,
            saturation: 0.55,
            lightness: 0.1,
            alpha: 1.0,
        }))
        .border_radius(BorderRadius::all(Val::Px(3.0)));

    style_builder
        .switch_context(Slider::HANDLE, None)
        .size(Val::Px(26.0))
        .border(UiRect::all(Val::Px(2.0)))
        .border_color(Color::Hsla(Hsla {
            hue: 34.0,
            saturation: 0.55,
            lightness: 0.1,
            alpha: 1.0,
        }))
        .border_radius(BorderRadius::all(Val::Px(13.0)))
        .animated()
        .background_color(AnimatedVals {
            idle: Color::Hsla(Hsla { hue: 34.0, saturation: 0.63, lightness: 0.55, alpha: 1.0 }),
            hover: Color::Hsla(Hsla { hue: 34.0, saturation: 0.7, lightness: 0.45, alpha: 1.0 }).into(),
            ..default()
        })
        .copy_from(theme_data.interaction_animation);
}

fn adjust_sickle_slider_theme(ui: &mut EntityCommands)
{
    let adjusted_theme = PseudoTheme::deferred_context(None, adjusted_slider_style);
    ui.insert(Theme::new(vec![adjusted_theme]));
}

//-------------------------------------------------------------------------------------------------------------------

fn setup_settings(mut c: Commands, mut s: ResMut<SceneLoader>)
{
    let scene = LoadableRef::new("ui.settings", "button_scene");
    c.ui_builder(UiRoot).load_scene(&mut s, scene, |l| {
        l.edit("button", |l| {
            l.on_pressed(|mut c: Commands| {
                c.react().broadcast(ToggleSettings);
            });
        });
    });
}

//-------------------------------------------------------------------------------------------------------------------

/// We have a separate reactor to do on/off because when the settings menu is triggered (e.g. by pressing Esc),
/// we don't know if it needs to be opened or closed - and the place where we trigger it shouldn't need to figure
/// that out.
fn handle_toggle_settings(
    mut state: Local<bool>,
    mut c: Commands,
    mut time: ResMut<Time<Virtual>>,
    powerup_buffer: Res<BufferedPowerUps>,
)
{
    let prev_state = *state;
    *state = !prev_state;
    match prev_state {
        true => {
            // Unpause time when settings closed, if we aren't in a powerup screen.
            // todo: this is a hacky solution, need a centralized time control system
            if !powerup_buffer.is_handling_powerup() {
                time.unpause();
            }

            c.react().broadcast(ToggleSettingsOff);
        }
        false => {
            // Pause time while in settings.
            time.pause();

            c.react().broadcast(ToggleSettingsOn);
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn spawn_settings_menu(mut c: Commands, mut s: ResMut<SceneLoader>, audio_settings: ReactRes<AudioSettings>)
{
    let scene = LoadableRef::new("ui.settings", "display_scene");
    c.ui_builder(UiRoot).load_scene(&mut s, scene, |l| {
        l.despawn_on_broadcast::<ToggleSettingsOff>();

        l.edit("window", |l| {
            // todo: controls image (non-configurable)

            l.edit("audio::slider", |l| {
                // Slider: sickle_ui built-in widget.
                let mut ui = l.slider(SliderConfig::horizontal(
                    None,
                    0.0,
                    100.0,
                    audio_settings.master_volume * 100.0,
                    true,
                ));
                let id = ui.id();
                let n = ui.update_on(entity_event::<SliderChanged>(id), |id| {
                    move |mut c: Commands, mut settings: ReactResMut<AudioSettings>, sliders: Query<&Slider>| {
                        let Ok(slider) = sliders.get(id) else { return };
                        settings.get_mut(&mut c).master_volume = slider.value() / 100.;
                    }
                });
                adjust_sickle_slider_theme(&mut n.entity_commands());
            });

            // todo: restart from day 1 button

            l.edit("footer::close_button", |l| {
                l.on_pressed(|mut c: Commands| {
                    c.react().broadcast(ToggleSettings);
                });
            });
        });
    });
}

//-------------------------------------------------------------------------------------------------------------------

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_systems(OnExit(GameState::Loading), setup_settings)
            .react(|rc| rc.on_persistent(broadcast::<ToggleSettings>(), handle_toggle_settings))
            .react(|rc| rc.on_persistent(broadcast::<ToggleSettingsOn>(), spawn_settings_menu))
            .add_systems(PostUpdate, detect_silder_change);
    }
}

//-------------------------------------------------------------------------------------------------------------------
