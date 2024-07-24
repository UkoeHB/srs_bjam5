use std::time::Duration;

use bevy::prelude::*;
use rand::Rng;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn update_transforms_for_attraction(
    mut buffer: Local<Vec<(usize, Entity, Transform)>>,
    mut c: Commands,
    clock: Res<GameClock>,
    mut pset: ParamSet<(
        Query<(Entity, &Transform, &AttractionSource)>,
        Query<(Entity, &mut Transform, &mut Attraction)>,
    )>,
)
{
    // Delta of this tick.
    let delta = clock.delta;

    // Collect attraction source locations.
    buffer.clear();
    for (entity, transform, source) in pset.p0().iter() {
        buffer.push((source.priority(), entity, *transform));
    }

    // Sort by source priority so high-priority sources are at the front of the vec.
    buffer.sort_unstable_by(|a, b| b.0.cmp(&a.0));

    // Update transforms of attracted entities.
    for (entity, mut transform, mut attraction) in pset.p1().iter_mut() {
        let Some((_, _, target_transform)) = buffer.iter().find(|(_, e, _)| *e == attraction.target) else {
            c.entity(entity).remove::<Attraction>();
            continue;
        };

        // Move the entity toward its attraction source.
        let distance = attraction.update_and_get_distance(delta);
        let direction =
            Dir3::new(target_transform.translation - transform.translation + attraction.target_offset.extend(0.))
                .map(|d| d.as_vec3())
                .unwrap_or_default();
        let movement = direction * distance;
        transform.translation += movement;
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn update_enemy_target_offsets(
    mut timer: ResMut<TargetOffsetUpdateTimer>,
    time: Res<Time>,
    mut enemies: Query<(&mut Attraction, &Transform), With<Mob>>,
    player: Query<&Transform, With<Player>>,
    mut rng: ResMut<GameRng>,
)
{
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return; // run every time the timer runs out
    }
    let rng = rng.rng();
    let player_transform = player.single();
    let max_offset_length = 75.;
    let precise_follow_dist = 75.; // when they will start following the player more precisely
    for (mut attraction, transform) in enemies.iter_mut() {
        // randomize offset, and clamp it to a set length
        attraction.target_offset =
            if (player_transform.translation - transform.translation).length() >= precise_follow_dist {
                Vec2::new(
                    rng.gen_range(-max_offset_length..=max_offset_length),
                    rng.gen_range(-max_offset_length..=max_offset_length),
                )
                .clamp_length_max(max_offset_length as f32)
            } else {
                Vec2::ZERO
            };
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Resource)]
pub struct TargetOffsetUpdateTimer(pub Timer);

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Debug)]
pub struct Attraction
{
    target: Entity,
    max_velocity_tps: f32,
    /// If this is set to zero then max velocity will be reached immediately.
    acceleration: f32,

    /// Cached
    current_vel: f32,

    // offset to make movement more random and bunch up less
    target_offset: Vec2,
}

impl Attraction
{
    pub fn new(target: Entity, max_velocity_tps: f32, acceleration: f32) -> Self
    {
        let current_vel = match acceleration {
            0. => max_velocity_tps,
            _ => 0.,
        };
        Self {
            target,
            max_velocity_tps,
            acceleration,
            current_vel,
            target_offset: Vec2::ZERO,
        }
    }

    pub fn target(&self) -> Entity
    {
        self.target
    }

    /// Updates internal velocity and calculates distance to travel this tick.
    pub fn update_and_get_distance(&mut self, delta: Duration) -> f32
    {
        let delta = delta.as_secs_f32();
        if self.current_vel < self.max_velocity_tps {
            self.current_vel += self.acceleration * delta;
            self.current_vel = self.current_vel.min(self.max_velocity_tps);
        }
        self.current_vel * delta
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Marker component indicating the current entity *might* be a target of attraction
/// for other entities.
///
/// Used to optimize attraction handling.
///
/// Set the priority based on the number of entities with Attraction that might target the entity.
#[derive(Component, Debug)]
pub enum AttractionSource
{
    LowPriority,
    MedPriority,
    HighPriority,
}

impl AttractionSource
{
    pub fn priority(&self) -> usize
    {
        match *self {
            Self::LowPriority => 0,
            Self::MedPriority => 1,
            Self::HighPriority => 2,
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(SystemSet, Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct AttractionUpdateSet;

//-------------------------------------------------------------------------------------------------------------------

pub struct AttractionPlugin;

impl Plugin for AttractionPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_systems(
            Update,
            (update_enemy_target_offsets, update_transforms_for_attraction)
                .chain()
                .in_set(AttractionUpdateSet)
                .run_if(in_state(GameState::Play)),
        )
        .insert_resource(TargetOffsetUpdateTimer(Timer::from_seconds(3., TimerMode::Repeating)));
    }
}

//-------------------------------------------------------------------------------------------------------------------
