use bevy::math::vec3;
use bevy::prelude::TransformSystem::TransformPropagate;
use bevy::prelude::*;
use bevy::sprite::{Anchor, MaterialMesh2dBundle};
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Debug)]
struct BillboardEntities
{
    tag: Entity,
    hp: Entity,
    exp: Entity,
}

impl Default for BillboardEntities
{
    fn default() -> Self
    {
        Self {
            tag: Entity::PLACEHOLDER,
            hp: Entity::PLACEHOLDER,
            exp: Entity::PLACEHOLDER,
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Component tracks which direction the player faces.
#[derive(Component, Debug, Copy, Clone, Eq, PartialEq, Default)]
pub enum PlayerDirection
{
    #[default]
    Up,
    Down,
    Left,
    Right,
    UpLeft,
    UpRight,
    DownRight,
    DownLeft,
}

impl PlayerDirection
{
    pub fn is_up(&self) -> bool
    {
        match self {
            Self::Up | Self::UpLeft | Self::UpRight => true,
            _ => false,
        }
    }

    pub fn is_down(&self) -> bool
    {
        match self {
            Self::Down | Self::DownLeft | Self::DownRight => true,
            _ => false,
        }
    }

    pub fn is_left(&self) -> bool
    {
        match self {
            Self::Left | Self::UpLeft | Self::DownLeft => true,
            _ => false,
        }
    }

    pub fn is_right(&self) -> bool
    {
        match self {
            Self::Right | Self::UpRight | Self::DownRight => true,
            _ => false,
        }
    }

    pub fn to_unit_vector(&self) -> Vec2
    {
        match self {
            Self::Up => Vec2 { x: 0., y: 1.0 },
            Self::Down => Vec2 { x: 0., y: -1.0 },
            Self::Left => Vec2 { x: -1.0, y: 0. },
            Self::Right => Vec2 { x: 1.0, y: 0. },
            Self::UpLeft => Vec2 { x: -0.70711, y: 0.70711 },
            Self::UpRight => Vec2 { x: 0.70711, y: 0.70711 },
            Self::DownRight => Vec2 { x: 0.70711, y: -0.70711 },
            Self::DownLeft => Vec2 { x: -0.70711, y: -0.70711 },
        }
    }
}

impl Into<Dir2> for PlayerDirection
{
    fn into(self) -> Dir2
    {
        Dir2::new_unchecked(self.to_unit_vector())
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Component tracks the current state of the player.
#[derive(Component, Debug, Copy, Clone, Eq, PartialEq)]
enum Action
{
    Standing,
    Running,
}

//-------------------------------------------------------------------------------------------------------------------

fn handle_key_inputs(direction: &mut PlayerDirection, action: &mut Action, next_direction: Option<PlayerDirection>)
{
    let next_action = match *action {
        Action::Standing | Action::Running => {
            *direction = next_direction.unwrap_or(*direction);

            match next_direction.is_some() {
                true => Action::Running,
                false => Action::Standing,
            }
        }
    };

    *action = next_action;
}

//-------------------------------------------------------------------------------------------------------------------

fn update_player_state_from_input(
    button_input: Res<ButtonInput<KeyCode>>,
    mut player: Query<(&mut PlayerDirection, &mut Action), With<Player>>,
    controls: ReactRes<Controls>,
)
{
    let (mut direction, mut action) = player.single_mut();
    let mut next_direction = None;

    if button_input.pressed(*controls.move_up)
        && !(direction.is_down() && button_input.pressed(*controls.move_down))
    {
        match next_direction {
            Some(PlayerDirection::Left) => {
                next_direction = Some(PlayerDirection::UpLeft);
            }
            Some(PlayerDirection::Right) => {
                next_direction = Some(PlayerDirection::UpRight);
            }
            _ => {
                next_direction = Some(PlayerDirection::Up);
            }
        }
    }
    if button_input.pressed(*controls.move_down) && !(direction.is_up() && button_input.pressed(*controls.move_up))
    {
        match next_direction {
            Some(PlayerDirection::Left) => {
                next_direction = Some(PlayerDirection::DownLeft);
            }
            Some(PlayerDirection::Right) => {
                next_direction = Some(PlayerDirection::DownRight);
            }
            _ => {
                next_direction = Some(PlayerDirection::Down);
            }
        }
    }
    if button_input.pressed(*controls.move_left)
        && !(direction.is_right() && button_input.pressed(*controls.move_right))
    {
        match next_direction {
            Some(PlayerDirection::Up) => {
                next_direction = Some(PlayerDirection::UpLeft);
            }
            Some(PlayerDirection::Down) => {
                next_direction = Some(PlayerDirection::DownLeft);
            }
            _ => {
                next_direction = Some(PlayerDirection::Left);
            }
        }
    }
    if button_input.pressed(*controls.move_right)
        && !(direction.is_left() && button_input.pressed(*controls.move_left))
    {
        match next_direction {
            Some(PlayerDirection::Up) => {
                next_direction = Some(PlayerDirection::UpRight);
            }
            Some(PlayerDirection::Down) => {
                next_direction = Some(PlayerDirection::DownRight);
            }
            _ => {
                next_direction = Some(PlayerDirection::Right);
            }
        }
    }

    handle_key_inputs(&mut direction, &mut action, next_direction);
}

//-------------------------------------------------------------------------------------------------------------------

fn update_player_transform_from_tick(
    time: Res<Time>,
    mut player: Query<(&mut Transform, &MoveSpeed, &PlayerDirection, &Action), With<Player>>,
)
{
    let (mut player_transform, speed, direction, action) = player.single_mut();
    let delta = time.delta();

    let translation_magnitude = match *action {
        Action::Standing => 0.,
        Action::Running => (speed.current() as f32) * delta.as_secs_f32(),
    };

    let translation_direction = direction.to_unit_vector();
    let translation = translation_direction * translation_magnitude;

    // Apply transform.
    player_transform.translation += translation.extend(0.);
}

//-------------------------------------------------------------------------------------------------------------------

fn update_player_animation(
    mut prev: Local<String>,
    mut c: Commands,
    constants: ReactRes<GameConstants>,
    animations: Res<SpriteAnimations>,
    player: Query<(Entity, &PlayerDirection, &Action), With<Player>>,
)
{
    let (player_entity, direction, action) = player.single();

    let anim_name = match *action {
        Action::Standing => match *direction {
            PlayerDirection::Up | PlayerDirection::UpLeft | PlayerDirection::UpRight => {
                &constants.player_standing_animation
            }
            PlayerDirection::Down => &constants.player_standing_animation,
            PlayerDirection::Left | PlayerDirection::DownLeft => &constants.player_standing_animation,
            PlayerDirection::Right | PlayerDirection::DownRight => &constants.player_standing_animation,
        },
        Action::Running => match *direction {
            PlayerDirection::Up | PlayerDirection::UpLeft | PlayerDirection::UpRight => {
                &constants.player_standing_animation
            }
            PlayerDirection::Down => &constants.player_standing_animation,
            PlayerDirection::Left | PlayerDirection::DownLeft => &constants.player_standing_animation,
            PlayerDirection::Right | PlayerDirection::DownRight => &constants.player_standing_animation,
        },
    };

    // Don't reset the animation if it stays the same.
    if *anim_name == *prev {
        return;
    }
    *prev = anim_name.clone();

    // Set the animation.
    c.entity(player_entity)
        .set_sprite_animation(&animations, anim_name);
}

//-------------------------------------------------------------------------------------------------------------------

fn update_player_billboard(
    constants: ReactRes<GameConstants>,
    mut e: TextEditor,
    player: Query<(&Health, &Level, &BillboardEntities), With<Player>>,
    mut transforms: Query<&mut Transform>,
)
{
    let Ok((hp, level, billboard)) = player.get_single() else { return };

    // Update level tag
    write_text!(e, billboard.tag, "{}", level.level());

    // Update exp bar
    if let Ok(mut transform) = transforms.get_mut(billboard.exp) {
        let scale = (level.exp() as f32) / level.exp_required().max(1.);
        transform.scale.x = scale;
        transform.translation.x = -(1. - scale) * constants.exp_bar_size.x / 2.;
    }

    // Update hp bar
    if let Ok(mut transform) = transforms.get_mut(billboard.hp) {
        let scale = (hp.current() as f32) / (hp.max().max(1) as f32);
        transform.scale.x = scale;
        transform.translation.x = -(1. - scale) * constants.hp_bar_size.x / 2.;
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn detect_player_death(mut c: Commands, mut events: EventReader<EntityDeath>, player: Query<Entity, With<Player>>)
{
    let player = player.single();
    if !events.read().any(|event| **event == player) {
        return;
    }
    c.react().broadcast(PlayerDied);
}

//-------------------------------------------------------------------------------------------------------------------

fn spawn_player(
    mut c: Commands,
    constants: ReactRes<GameConstants>,
    animations: Res<SpriteAnimations>,
    billboard_cache: Res<BillboardCache>,
    fonts: Res<FontMap>,
)
{
    let mut billboard_entities = BillboardEntities::default();
    c.spawn((
        Player,
        (
            Health::new(constants.player_base_hp),
            HealthRegen::new(0),
            Armor::new(constants.player_base_armor),
            CooldownReduction::new(0),
            MoveSpeed::new(constants.player_run_speed_tps),
            CollectionRange::new(constants.hoover_detection_range),
            AreaSize::new(1.0),
            DamageAmp::new(0),
            ExpAmp::new(0),
            Level::new(constants.player_exp_start, constants.player_exp_rate),
        ),
        SpatialBundle::from_transform(Transform::default()),
        SpriteLayer::Objects,
        PlayerDirection::Up,
        Action::Standing,
        AabbSize(constants.player_size),
        AttractionSource::HighPriority,
        StateScoped(GameState::Play),
        BoundInMap,
    ))
    .set_sprite_animation(&animations, &constants.player_standing_animation)
    .with_children(|cb| {
        // Player level
        let tag_translation = vec3(
            -(constants.hp_bar_size.x / 2.) + constants.level_tag_offset.x,
            constants.player_size.y / 2. + constants.level_tag_offset.y,
            0.,
        );
        billboard_entities.tag = cb
            .spawn((
                Text2dBundle {
                    text: Text::from_section(
                        "1",
                        TextStyle {
                            font: fonts.get(&constants.level_tag_font),
                            font_size: constants.level_tag_font_size,
                            color: constants.level_tag_color,
                        },
                    ),
                    text_anchor: Anchor::CenterRight,
                    transform: Transform { translation: tag_translation, scale: Vec3::ONE, ..default() },
                    ..default()
                },
                SpriteLayer::PlayerBillboardLv1,
            ))
            .id();

        // Player exp bar
        let exp_bar_translation = vec3(0., constants.player_size.y / 2. + constants.exp_bar_offset, 0.);
        cb.spawn((
            MaterialMesh2dBundle {
                mesh: billboard_cache.exp_bar_mesh().into(),
                material: billboard_cache.exp_bar_empty_color(),
                transform: Transform { translation: exp_bar_translation, ..default() },
                ..default()
            },
            SpriteLayer::PlayerBillboardLv1,
        ))
        .with_children(|cb| {
            billboard_entities.exp = cb
                .spawn((
                    MaterialMesh2dBundle {
                        mesh: billboard_cache.exp_bar_mesh().into(),
                        material: billboard_cache.exp_bar_filled_color(),
                        transform: Transform { scale: vec3(0., 1., 1.), ..default() },
                        ..default()
                    },
                    Anchor::CenterRight,
                    SpriteLayer::PlayerBillboardLv2,
                ))
                .id();
        });

        // Player health on top of exp bar
        let hp_bar_translation = vec3(
            0.,
            constants.player_size.y / 2.
                + constants.exp_bar_offset
                + constants.exp_bar_size.y
                + constants.hp_bar_offset,
            0.,
        );
        cb.spawn((
            MaterialMesh2dBundle {
                mesh: billboard_cache.hp_bar_mesh().into(),
                material: billboard_cache.hp_bar_empty_color(),
                transform: Transform { translation: hp_bar_translation, ..default() },
                ..default()
            },
            SpriteLayer::PlayerBillboardLv1,
        ))
        .with_children(|cb| {
            billboard_entities.hp = cb
                .spawn((
                    MaterialMesh2dBundle {
                        mesh: billboard_cache.hp_bar_mesh().into(),
                        material: billboard_cache.hp_bar_filled_color(),
                        transform: Transform { scale: Vec3::ONE, ..default() },
                        ..default()
                    },
                    SpriteLayer::PlayerBillboardLv2,
                ))
                .id();
        });
    })
    .insert(billboard_entities);
}

//-------------------------------------------------------------------------------------------------------------------

/// Marker component for player entities.
#[derive(Component, Debug)]
pub struct Player;

//-------------------------------------------------------------------------------------------------------------------

#[derive(SystemSet, Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct PlayerUpdateSet;

//-------------------------------------------------------------------------------------------------------------------

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_systems(OnEnter(GameState::Play), spawn_player)
            .add_systems(
                Update,
                (
                    update_player_state_from_input,
                    update_player_transform_from_tick,
                    update_player_animation,
                )
                    .chain()
                    .in_set(PlayerUpdateSet),
            )
            .add_systems(Update, detect_player_death.in_set(DamageSet::HandleDeaths))
            .add_systems(
                PostUpdate,
                update_player_billboard
                    .before(TransformPropagate)
                    .run_if(in_state(PlayState::Day)),
            );
    }
}

//-------------------------------------------------------------------------------------------------------------------
