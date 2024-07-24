use bevy::math::bounding::{Aabb2d, IntersectsVolume};
use bevy::prelude::*;
use bevy_cobweb::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn handle_collectable_collisions(
    mut c: Commands,
    mut player: Query<(&mut Level, &mut Health, &Transform, &AabbSize), With<Player>>,
    mut karma: ReactResMut<Karma>,
    collectables: Query<(Entity, &Collectable, &Transform, &AabbSize)>,
    mut powerups: ResMut<BufferedPowerUps>,
)
{
    let Ok((mut level, mut health, player_transform, player_size)) = player.get_single_mut() else { return };
    let player_aabb = Aabb2d::new(player_transform.translation.truncate(), **player_size / 2.);

    for (entity, collectable, collectable_transform, collectable_size) in collectables.iter() {
        // Check for collision.
        let entity_aabb = Aabb2d::new(collectable_transform.translation.truncate(), **collectable_size / 2.);
        if !entity_aabb.intersects(&player_aabb) {
            continue;
        }

        // Handle type.
        match *collectable {
            Collectable::Exp(exp) => {
                let levels = level.add_exp(exp);
                powerups.insert(levels.iter().map(|_| PowerUpSource::LevelUp));
            }
            Collectable::Karma(k) => {
                karma.get_mut(&mut c).add(k);
            }
            Collectable::HealthPack(hp) => {
                health.add(hp);
            }
        }

        // Despawn entity now it has been collected.
        c.entity(entity).despawn_recursive();
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Adds Attraction to collectables in-range that don't have Attraction yet.
///
/// Do this after detection collisions to reduce redundant query accesses (tiny perf win).
fn handle_collectable_detection(
    mut c: Commands,
    constants: ReactRes<GameConstants>,
    player: Query<(Entity, &Transform, &AabbSize), With<Player>>,
    collectables: Query<(Entity, &Collectable, &Transform, &AabbSize), Without<Attraction>>,
)
{
    let Ok((player_entity, player_transform, player_size)) = player.get_single() else { return };
    let player_aabb = Aabb2d::new(player_transform.translation.truncate(), **player_size / 2.);

    for (entity, collectable, collectable_transform, collectable_size) in collectables.iter() {
        // Get collectable's detection range if allowed.
        let Some(detection_range) = collectable.get_detection_range(&constants, **collectable_size) else {
            continue;
        };

        // Check for collision with the collectable's detection range.
        let entity_aabb = Aabb2d::new(collectable_transform.translation.truncate(), detection_range / 2.);
        if !entity_aabb.intersects(&player_aabb) {
            continue;
        }

        // Add attraction.
        c.entity(entity).try_insert(Attraction::new(
            player_entity,
            constants.collectable_max_vel,
            constants.collectable_accel,
        ));
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Component that contains an entity's size for bounding-box intersections.
#[derive(Component, Deref, DerefMut)]
pub struct AabbSize(pub Vec2);

//-------------------------------------------------------------------------------------------------------------------

#[derive(SystemSet, Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct IntersectionsUpdateSet;

//-------------------------------------------------------------------------------------------------------------------

pub struct IntersectionsPlugin;

impl Plugin for IntersectionsPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_systems(
            Update,
            (handle_collectable_collisions, handle_collectable_detection)
                .chain()
                .in_set(IntersectionsUpdateSet),
        );
    }
}

//-------------------------------------------------------------------------------------------------------------------
