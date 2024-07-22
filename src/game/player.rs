use bevy::math::bounding::{Aabb2d, IntersectsVolume};
use bevy::prelude::*;
use bevy_cobweb::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Component tracks which direction the player faces.
#[derive(Component, Debug, Copy, Clone, Eq, PartialEq)]
enum PlayerDirection
{
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
    constants: ReactRes<GameConstants>,
    time: Res<Time>,
    mut player: Query<(&mut Transform, &PlayerDirection, &Action, &AabbSize), With<Player>>,
    barriers: Query<(&Transform, &AabbSize), (With<Barrier>, Without<Player>)>,
)
{
    let (mut transform, direction, action, player_size) = player.single_mut();
    let delta = time.delta();

    let translation_magnitude = match *action {
        Action::Standing => 0.,
        Action::Running => constants.player_run_speed_tps * delta.as_secs_f32(),
    };

    let translation_direction = direction.to_unit_vector();
    let translation = translation_direction * translation_magnitude;
    let target_translation = transform.translation.truncate() + translation;

    // Check if transform is valid.
    let player_bb = Aabb2d::new(target_translation, **player_size / 2.);

    for (transform, size) in barriers.iter() {
        let entity_aabb = Aabb2d::new(transform.translation.truncate(), **size / 2.);
        if entity_aabb.intersects(&player_bb) {
            // Reject player movement if movement will intersect with a barrier.
            // NOTE: This is a hacky solution because exactly where a player 'stops' depends on their framerate.
            return;
        }
    }

    // Check if the transform moves the player outside the map area.
    let map_bb = Aabb2d::new(Vec2::default(), map_area_half_size(&constants));
    if !player_bb.intersects(&map_bb) {
        return;
    }

    // Apply transform.
    transform.translation += translation.extend(0.);
}

//-------------------------------------------------------------------------------------------------------------------

fn update_player_animation(
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

    c.entity(player_entity)
        .set_sprite_animation(&animations, anim_name);
}

//-------------------------------------------------------------------------------------------------------------------

fn spawn_player(mut c: Commands, constants: ReactRes<GameConstants>, animations: Res<SpriteAnimations>)
{
    c.spawn((
        Player { health: constants.player_base_hp },
        SpatialBundle::from_transform(Transform::default()),
        SpriteLayer::Objects,
        PlayerDirection::Up,
        Action::Standing,
        AabbSize(constants.player_size),
        //todo: scoping to GameState::Play means the player despawns on entering GameState::DayOver, even though
        // we may want to continue displaying the player in the background
        StateScoped(GameState::Play),
    ))
    .set_sprite_animation(&animations, &constants.player_standing_animation);
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Debug)]
pub struct Player
{
    pub health: usize,
}

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
            );
    }
}

//-------------------------------------------------------------------------------------------------------------------