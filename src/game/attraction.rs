use std::time::Duration;

use bevy::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn update_transforms_for_attraction(
    mut buffer: Local<Vec<(usize, Entity, Transform)>>,
    mut c: Commands,
    clock: Res<GameClock>,
    mut pset: ParamSet<(
        Query<(Entity, &Transform, &AttractionSource)>,
        Query<(Entity, &mut Transform, &mut Attraction, &mut Sprite)>,
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
    for (entity, mut transform, mut attraction, mut sprite) in pset.p1().iter_mut() {
        let Some((_, _, target_transform)) = buffer.iter().find(|(_, e, _)| *e == attraction.target) else {
            c.entity(entity).remove::<Attraction>();
            continue;
        };
        let initial_vector = target_transform.translation - transform.translation;

        let vector = if initial_vector.length() > attraction.stop_distance {
            attraction.set_is_stopped(false);

            let target_offset = attraction.target_offset.clamp_length(
                attraction.stop_distance,
                attraction.stop_distance.max(initial_vector.length() / 2.),
            );

            initial_vector + target_offset.extend(0.)
        } else {
            attraction.set_is_stopped(true);
            Vec3::default()
        };

        sprite.flip_x = attraction.auto_flip_sprite && initial_vector.x < 0.;

        // Move the entity toward its attraction source.
        let distance = attraction
            .update_and_get_distance(delta)
            .min(vector.length());
        let direction = vector.normalize_or(Vec3::default());
        let movement = direction * distance;
        transform.translation += movement;
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Debug)]
pub struct Attraction
{
    target: Entity,
    max_velocity_tps: f32,
    /// If this is set to zero then max velocity will be reached immediately.
    acceleration: f32,
    /// Offset to make movement more random and bunch up less.
    target_offset: Vec2,
    /// How close to get before stopping, prevents jitter and can be used to make an entity sit at a certain
    /// range.
    stop_distance: f32,

    /// Cached
    current_vel: f32,
    is_stopped: bool,
    auto_flip_sprite: bool,
}

impl Attraction
{
    pub fn new(
        target: Entity,
        max_velocity_tps: f32,
        acceleration: f32,
        target_offset: Vec2,
        stop_distance: f32,
        auto_flip_sprite: bool,
    ) -> Self
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
            target_offset,
            stop_distance,
            is_stopped: false,
            auto_flip_sprite,
        }
    }

    pub fn target(&self) -> Entity
    {
        self.target
    }

    pub fn is_stopped(&self) -> bool
    {
        self.is_stopped
    }

    /// Updates internal velocity and calculates distance to travel this tick.
    fn update_and_get_distance(&mut self, delta: Duration) -> f32
    {
        let delta = delta.as_secs_f32();
        if self.current_vel < self.max_velocity_tps {
            self.current_vel += self.acceleration * delta;
            self.current_vel = self.current_vel.min(self.max_velocity_tps);
        }
        self.current_vel * delta
    }

    fn set_is_stopped(&mut self, stopped: bool)
    {
        self.is_stopped = stopped
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
        app.add_systems(Update, update_transforms_for_attraction.in_set(AttractionUpdateSet));
    }
}

//-------------------------------------------------------------------------------------------------------------------
