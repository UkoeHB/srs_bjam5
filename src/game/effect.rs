use std::f32::consts::PI;
use std::marker::PhantomData;
use std::time::Duration;

use bevy::math::bounding::{AabbCast2d, BoundingVolume, IntersectsVolume};
use bevy::prelude::*;

//use bevy_cobweb::prelude::*;
use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn apply_effect_zones<T: Component>(
    mut c: Commands,
    mut zones: Query<(Entity, &mut EffectZone<T>, &AabbSize, &Transform, Option<&PrevLocation>), Without<T>>,
    targets: Query<(Entity, &AabbSize, &Transform), (With<T>, Without<EffectZone<T>>)>,
    clock: Res<GameClock>,
)
{
    let time = clock.elapsed;

    for (zone_entity, mut zone, zone_aabb, zone_transform, maybe_zone_last_pos) in zones.iter_mut() {
        // Check current core cooldown.
        if let Some(next) = zone.next_effect_time {
            if time < next {
                continue;
            }
        }

        // Apply effect zone.
        // - We correct effect zones for their rotation, mainly so projectile intersections make more sense.
        // - We also extend effect zones based on their position last tick, so fast-moving zones don't skip over
        //   entities.
        zone.next_effect_time = None;
        let mut rotation = Quat::default().angle_between(zone_transform.rotation.normalize());
        if rotation > PI / 2. {
            rotation = PI - rotation;
        }
        rotation = rotation.clamp(0., PI / 2.);
        let entity_aabb = zone_aabb
            .get_2d_from_vec(Vec2::default())
            .transformed_by(zone_transform.translation.truncate(), rotation);
        let last_pos = *maybe_zone_last_pos
            .cloned()
            .unwrap_or(PrevLocation(zone_transform.translation.truncate()));
        let entity_aabb = AabbCast2d::new(
            entity_aabb,
            Vec2::default(),
            Dir2::new(last_pos - zone_transform.translation.truncate())
                .unwrap_or(Dir2::new_unchecked(Vec2::default().with_x(1.))),
            (zone_transform.translation.truncate() - last_pos).length(),
        );

        match zone.config {
            EffectZoneConfig::Target { target, cooldown_ms } => {
                // Check intersection with target.
                let Ok((target, aabb, transform)) = targets.get(target) else { continue };
                let target_aabb = aabb.get_2d(transform);
                if !entity_aabb.intersects(&target_aabb) {
                    continue;
                }

                // Apply effect.
                (zone.callback)(zone_entity, target, &mut c);

                // Set cooldown.
                zone.next_effect_time = Some(time + Duration::from_millis(cooldown_ms as u64));
            }
            EffectZoneConfig::SelfDestructSingle => {
                // Check intersection with any targets.
                let mut count = 0;
                for (target, aabb, transform) in targets.iter() {
                    // Check intersection.
                    let target_aabb = aabb.get_2d(transform);
                    if !entity_aabb.intersects(&target_aabb) {
                        continue;
                    }

                    // Apply effect.
                    (zone.callback)(zone_entity, target, &mut c);

                    // Exit now that a single target has been found.
                    count += 1;
                    break;
                }

                if count == 0 {
                    continue;
                }

                // Self-destruct.
                c.entity(zone_entity).despawn_recursive();

                // Set cooldown (for sanity).
                zone.next_effect_time = Some(time + Duration::from_millis(1_000_000));
            }
            EffectZoneConfig::SelfDestruct => {
                // Check intersection with any targets.
                let mut count = 0;
                for (target, aabb, transform) in targets.iter() {
                    // Check intersection.
                    let target_aabb = aabb.get_2d(transform);
                    if !entity_aabb.intersects(&target_aabb) {
                        continue;
                    }

                    // Apply effect.
                    (zone.callback)(zone_entity, target, &mut c);

                    count += 1;
                }

                if count == 0 {
                    continue;
                }

                // Self-destruct.
                c.entity(zone_entity).despawn_recursive();

                // Set cooldown (for sanity).
                zone.next_effect_time = Some(time + Duration::from_millis(1_000_000));
            }
            EffectZoneConfig::ApplyAndRegenSingle { cooldown_ms } => {
                // Check intersection with any targets.
                let mut count = 0;
                for (target, aabb, transform) in targets.iter() {
                    // Check intersection.
                    let target_aabb = aabb.get_2d(transform);
                    if !entity_aabb.intersects(&target_aabb) {
                        continue;
                    }

                    // Apply effect.
                    (zone.callback)(zone_entity, target, &mut c);

                    // Exit now that a single target has been found.
                    count += 1;
                    break;
                }

                if count == 0 {
                    continue;
                }

                // Set cooldown.
                zone.next_effect_time = Some(time + Duration::from_millis(cooldown_ms as u64));
            }
            EffectZoneConfig::ApplyAndRegen { cooldown_ms } => {
                // Check intersection with any targets.
                let mut count = 0;
                for (target, aabb, transform) in targets.iter() {
                    // Check intersection.
                    let target_aabb = aabb.get_2d(transform);
                    if !entity_aabb.intersects(&target_aabb) {
                        continue;
                    }

                    // Apply effect.
                    (zone.callback)(zone_entity, target, &mut c);

                    count += 1;
                }

                if count == 0 {
                    continue;
                }

                // Set cooldown.
                zone.next_effect_time = Some(time + Duration::from_millis(cooldown_ms as u64));
            }
            EffectZoneConfig::Continuous { cooldown_ms } => {
                // Check intersection with any targets.
                for (target, aabb, transform) in targets.iter() {
                    // Check intersection.
                    let target_aabb = aabb.get_2d(transform);
                    if !entity_aabb.intersects(&target_aabb) {
                        continue;
                    }

                    // Find entry and check cooldown.
                    // - We set flag to `true` so we know an intersection was found this tick.
                    if let Some((_, next, flag)) = zone
                        .target_cooldowns
                        .iter_mut()
                        .find(|(entity, ..)| *entity == target)
                    {
                        *flag = true;
                        if time < *next {
                            // Cooldown in effect, wait more.
                            continue;
                        }
                        *next = time + Duration::from_millis(cooldown_ms as u64);
                    } else {
                        zone.target_cooldowns.push((
                            target,
                            time + Duration::from_millis(cooldown_ms as u64),
                            true,
                        ));
                    }

                    // Apply effect.
                    (zone.callback)(zone_entity, target, &mut c);
                }

                // Clean up non-intersected entities.
                zone.target_cooldowns.retain_mut(|(_, _, flag)| {
                    let intersected = *flag;
                    *flag = false;
                    intersected
                });
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub enum EffectZoneConfig
{
    /// Applies effect to intersected target, then despawns self.
    ///
    /// If the target is despawned, then the effect will be ignored. The user needs to handle that case
    /// separately.
    ///
    /// Does not go on cooldown if no intersected target.
    Target
    {
        target: Entity, cooldown_ms: u64
    },
    /// Applies effect to one intersected entity, then despawns self.
    ///
    /// Does nothing until there is at least one intersected entity.
    SelfDestructSingle,
    /// Applies effect to intersected entities, then despawns self.
    ///
    /// Does nothing until there is at least one intersected entity.
    SelfDestruct,
    /// Applies effect to one intersected entity, then hides self until cooldown ends.
    ///
    /// Does not go on cooldown if no intersected entities.
    ApplyAndRegenSingle
    {
        cooldown_ms: u64
    },
    /// Applies effect to intersected entities, then hides self until cooldown ends.
    ///
    /// Does not go on cooldown if no intersected entities.
    ApplyAndRegen
    {
        cooldown_ms: u64
    },
    /// Applies effect to intersected entities, and tracks cooldowns per-entity.
    ///
    /// The per-entity cooldown will reset if the effect zone stops intersecting with the target.
    Continuous
    {
        /// Cooldown per intersected enemy.
        cooldown_ms: u64,
    },
}

//-------------------------------------------------------------------------------------------------------------------

/// Effect zone parameterized by the component the zone should use to identify enemy entities.
#[derive(Component)]
pub struct EffectZone<E: Component>
{
    config: EffectZoneConfig,
    /// Callback invoked when the effect is applied.
    ///
    /// Note that an effect can be composed of multiple mutations, e.g. damage + knockback.
    ///
    /// Signature: fn(this entity, target entity, world)
    callback: fn(Entity, Entity, &mut Commands),

    /// Buffered data
    next_effect_time: Option<Duration>,
    /// [ (target, next time effect can be applied) ]
    target_cooldowns: Vec<(Entity, Duration, bool)>,

    _p: PhantomData<E>,
}

impl<E: Component> EffectZone<E>
{
    pub fn new(config: EffectZoneConfig, callback: fn(Entity, Entity, &mut Commands)) -> Self
    {
        Self {
            config,
            callback,
            next_effect_time: None,
            target_cooldowns: Vec::default(),
            _p: PhantomData::default(),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(SystemSet, Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct EffectUpdateSet;

//-------------------------------------------------------------------------------------------------------------------

pub trait RegisterEffectRelationshipAppExt
{
    /// Registers system to apply effect zones.
    ///
    /// `T` is a component marker to identify entities that can be affected by this zone.
    fn add_effect_target<T: Component>(&mut self) -> &mut Self;
}

impl RegisterEffectRelationshipAppExt for App
{
    fn add_effect_target<T: Component>(&mut self) -> &mut Self
    {
        self.add_systems(Update, apply_effect_zones::<T>.in_set(EffectUpdateSet));
        self
    }
}

//-------------------------------------------------------------------------------------------------------------------
