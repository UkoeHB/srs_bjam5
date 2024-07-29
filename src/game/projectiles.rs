use std::time::Duration;

use bevy::ecs::system::EntityCommands;
use bevy::math::bounding::Aabb2d;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_spritesheet_animation::animation::AnimationId;
use serde::{Deserialize, Serialize};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn add_effect_animation(ec: &mut EntityCommands, projectile: &Projectile, transform: &Transform)
{
    let Some(animation) = projectile.effect_animation else { return };

    ec.insert((
        SpatialBundle::from_transform(*transform),
        StateScoped(GameState::Play),
        DespawnOnAnimationCycle,
        projectile
            .effect_sprite_layer
            .unwrap_or(SpriteLayer::Projectiles),
    ))
    .set_sprite_animation_from_id(animation);
}

//-------------------------------------------------------------------------------------------------------------------

fn apply_projectile_effect_impl<T: Component>(
    In((projectile, target)): In<(Entity, Entity)>,
    mut c: Commands,
    mut events: EventWriter<DamageEvent>,
    projectiles: Query<(&Transform, &Projectile)>,
)
{
    let Ok((transform, projectile)) = projectiles.get(projectile) else { return };

    match projectile.projectile_type {
        ProjectileType::SingleUse { damage } | ProjectileType::Continuous { damage, .. } => {
            events.send(DamageEvent { source: projectile.source, target, damage });
            if projectile.effect_animation.is_some() {
                let mut ec = c.spawn_empty();
                add_effect_animation(&mut ec, projectile, transform);
            }
        }
        ProjectileType::Pulse { damage, area, .. } => {
            let mut ec = c.spawn((
                EffectZone::<T>::new(
                    // Use regen so the effect isn't despawned. We want it to despawn after the animation.
                    EffectZoneConfig::ApplyAndRegen { cooldown_ms: 1_000_000 },
                    apply_collider_effect,
                ),
                Collider { damage },
                AabbSize(area),
            ));

            add_effect_animation(&mut ec, projectile, transform);
        }
        ProjectileType::Explosion { damage, area } => {
            let mut ec = c.spawn((
                EffectZone::<T>::new(
                    // Use regen so the effect isn't despawned. We want it to despawn after the animation.
                    EffectZoneConfig::ApplyAndRegen { cooldown_ms: 1_000_000 },
                    apply_collider_effect,
                ),
                Collider { damage },
                AabbSize(area),
            ));

            add_effect_animation(&mut ec, projectile, transform);
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn apply_projectile_effect<T: Component>(projectile: Entity, target: Entity, c: &mut Commands)
{
    c.syscall((projectile, target), apply_projectile_effect_impl::<T>);
}

//-------------------------------------------------------------------------------------------------------------------

fn update_projectile_transforms(
    mut c: Commands,
    clock: Res<GameClock>,
    constants: ReactRes<GameConstants>,
    mut projectiles: Query<(Entity, &mut Transform, &Projectile)>,
)
{
    let time = clock.elapsed;
    let delta_secs = clock.delta.as_secs_f32();
    let map_bb = Aabb2d::new(
        Vec2::default(),
        Vec2 {
            x: constants.map_size.x as f32 * constants.map_tile_size.x / 2. + 50.,
            y: constants.map_size.y as f32 * constants.map_tile_size.y / 2. + 50.,
        },
    );

    for (entity, mut transform, projectile) in projectiles.iter_mut() {
        // Kill the projectile if it ran out of lifetime.
        if let Some(despawn_time) = projectile.despawn_time {
            if despawn_time < time {
                c.entity(entity).despawn_recursive();
                continue;
            }
        }

        // Move the projectile.
        let distance = projectile.direction * projectile.velocity_tps * delta_secs;
        transform.translation += distance.extend(0.);

        // Kill the projectile if it's off-map.
        let pos = transform.translation.truncate();
        if map_bb.closest_point(pos) != pos {
            c.entity(entity).despawn_recursive();
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Reflect, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProjectileType
{
    SingleUse
    {
        damage: usize
    },
    Continuous
    {
        damage: usize, cooldown_ms: u64
    },
    /// Note: the projectile's `effect_animation` field must be set to use this.
    Pulse
    {
        cooldown_ms: u64, damage: usize, area: Vec2
    },
    /// Note: the projectile's `effect_animation` field must be set to use this.
    Explosion
    {
        damage: usize, area: Vec2
    },
}

impl ProjectileType
{
    pub fn with_area_size(mut self, area_size: &AreaSize) -> Self
    {
        match &mut self {
            Self::Pulse { area, .. } | Self::Explosion { area, .. } => {
                *area = area_size.calculate_area(*area);
            }
            _ => (),
        }
        self
    }
}

impl Default for ProjectileType
{
    fn default() -> Self
    {
        Self::SingleUse { damage: 0 }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Debug)]
pub struct Projectile
{
    source: Entity,
    projectile_type: ProjectileType,
    effect_animation: Option<AnimationId>,
    effect_sprite_layer: Option<SpriteLayer>,
    velocity_tps: f32,
    direction: Dir2,
    despawn_time: Option<Duration>,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectileConfig
{
    pub projectile_type: ProjectileType,
    /// You can set this to zero if you want to leave a 'splotch' on the ground that acts like poison
    /// (set the `max_lifetime` field).
    /// - Note/todo: to get a 'homing' effect you'd need to integrate projectiles with attraction so the
    ///   projectile sprite's orientation can be adjusted (maybe update attraction to optionally adjust
    ///   rotation?).
    pub velocity_tps: f32,
    /// The animation is assumed to represent the projectile moving along the +x axis.
    ///
    /// It will be flipped when moving in the -x direction, and will be rotated to follow the direction of travel.
    pub animation: String,
    /// Size of the projectile, used for collision tests.
    pub size: Vec2,
    /// Layer for the main animation. Defaults to `SpriteLayer::Projectiles`.
    #[reflect(default)]
    pub sprite_layer: Option<SpriteLayer>,
    /// Optional lifetime for deciding when to auto-despawn the projectile. Can be used for slow/non-moving
    /// projectiles that apply damage over time effects.
    #[reflect(default)]
    pub max_lifetime_ms: Option<u64>,
    /// Animation that plays when the projectile's effect is applied.
    #[reflect(default)]
    pub effect_animation: Option<String>,
    /// Defaults to `SpriteLayer::Projectiles`.
    #[reflect(default)]
    pub effect_sprite_layer: Option<SpriteLayer>,
}

impl ProjectileConfig
{
    pub fn create_projectile<T: Component>(
        &self,
        c: &mut Commands,
        clock: &GameClock,
        animations: &SpriteAnimations,
        source: Entity,
        spawn_location: Vec2,
        direction: Dir2,
        area_size: &AreaSize,
        custom_applier: Option<fn(Entity, Entity, &mut Commands)>,
    ) -> Option<Entity>
    {
        let applier = custom_applier.unwrap_or(apply_projectile_effect::<T>);
        let effect_zone = match self.projectile_type {
            ProjectileType::SingleUse { .. } => EffectZone::<T>::new(EffectZoneConfig::SelfDestruct, applier),
            ProjectileType::Continuous { cooldown_ms, .. } => {
                EffectZone::<T>::new(EffectZoneConfig::Continuous { cooldown_ms }, applier)
            }
            ProjectileType::Pulse { cooldown_ms, .. } => {
                if self.effect_animation.is_none() {
                    tracing::error!("failed creating pulse projectile with animation {:?}; effect_animation \
                        field is required but not set", self.animation);
                    return None;
                }

                EffectZone::<T>::new(EffectZoneConfig::ApplyAndRegenSingle { cooldown_ms }, applier)
            }
            ProjectileType::Explosion { .. } => {
                if self.effect_animation.is_none() {
                    tracing::error!("failed creating explosion projectile with animation {:?}; effect_animation \
                        field is required but not set", self.animation);
                    return None;
                }

                EffectZone::<T>::new(EffectZoneConfig::SelfDestructSingle, applier)
            }
        };

        let effect_animation = self
            .effect_animation
            .as_ref()
            .map(|a| animations.get_id(a))
            .flatten();

        let rotation = Quat::from_rotation_z(direction.rotation_from_x().as_radians());
        //let flip_x = direction.x < 0.;
        let flip_x = false;
        let size = area_size.calculate_area(self.size);
        let scale = Vec3 { x: size.x / self.size.x, y: size.y / self.size.y, z: 1.0 };

        Some(
            c.spawn((
                effect_zone,
                Projectile {
                    source,
                    projectile_type: self.projectile_type.clone().with_area_size(area_size),
                    effect_animation,
                    effect_sprite_layer: self.effect_sprite_layer,
                    velocity_tps: self.velocity_tps,
                    direction,
                    despawn_time: self
                        .max_lifetime_ms
                        .map(|l| clock.elapsed + Duration::from_millis(l)),
                },
                StateScoped(GameState::Play),
                AabbSize(size),
                PrevLocation(spawn_location),
                SpatialBundle::from_transform(
                    Transform::from_translation(spawn_location.extend(0.))
                        .with_rotation(rotation)
                        .with_scale(scale),
                ),
                Sprite { flip_x, ..default() },
                self.sprite_layer.unwrap_or(SpriteLayer::Projectiles),
            ))
            .set_sprite_animation(animations, &self.animation)
            .id(),
        )
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(SystemSet, Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct ProjectileUpdateSet;

//-------------------------------------------------------------------------------------------------------------------

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_event::<DamageEvent>()
            .add_systems(Update, update_projectile_transforms.in_set(ProjectileUpdateSet));
    }
}

//-------------------------------------------------------------------------------------------------------------------
