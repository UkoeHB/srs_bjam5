use bevy::math::bounding::IntersectsVolume;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Finds dead entities with `CollectableDrop` and spawns the drop.
///
/// Only considers entities that emit `EntityDeath` events. If a new kind of droppable entity is introduced then
/// that must be handled separately.
fn handle_collectable_drops(
    mut deaths: EventReader<EntityDeath>,
    mut c: Commands,
    mut rng: ResMut<GameRng>,
    images: Res<ImageMap>,
    constants: ReactRes<GameConstants>,
    drops: Query<(&CollectableDrop, &Transform)>,
)
{
    let rng = rng.rng();
    let radius = constants.drop_radius;

    for (drop, transform) in deaths.read().filter_map(|death| drops.get(**death).ok()) {
        let location = transform.translation.truncate();

        for collectable in drop.iter() {
            // Select random nearby location to drop it.
            let offset = Vec2 {
                x: rng.gen_range(-radius..radius),
                y: rng.gen_range(-radius..radius),
            }
            .clamp_length(0., radius);
            let location = location + offset;

            // Drop it.
            collectable.spawn(&mut c, &constants, &images, location);
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn apply_collectable_effect_impl(
    In((collectable, _player_entity)): In<(Entity, Entity)>,
    collectables: Query<&Collectable>,
    mut c: Commands,
    constants: ReactRes<GameConstants>,
    mut player: Query<(&mut Level, &mut Health, &ExpAmp), With<Player>>,
    mut karma: ReactResMut<Karma>,
    mut powerups: ResMut<BufferedPowerUps>,
)
{
    let Ok((mut level, mut health, exp_amp)) = player.get_single_mut() else { return };
    let Ok(collectable) = collectables.get(collectable) else { return };

    // Handle type.
    match *collectable {
        Collectable::Exp(exp) => {
            let levels = level.add_exp(exp, &exp_amp);
            powerups.insert(levels.iter().map(|_| PowerupSource::LevelUp));
        }
        Collectable::Karma(k) => {
            karma.get_mut(&mut c).add(k);
        }
        Collectable::HealthPack => {
            let hp = (constants.collectable_hp_max_health * (health.max() as f32)).round() as usize;
            health.add(hp);
        }
    }
}

fn apply_collectable_effect(source: Entity, target: Entity, c: &mut Commands)
{
    c.syscall((source, target), apply_collectable_effect_impl);
}

//-------------------------------------------------------------------------------------------------------------------

/// Adds Attraction to collectables in-range that don't have Attraction yet.
fn handle_collectable_detection(
    mut c: Commands,
    constants: ReactRes<GameConstants>,
    player: Query<(Entity, &CollectionRange, &Transform, &AabbSize), With<Player>>,
    collectables: Query<(Entity, &Collectable, &Transform, &AabbSize), Without<Attraction>>,
)
{
    let Ok((player_entity, range, player_transform, player_size)) = player.get_single() else { return };
    let player_aabb = player_size.get_2d(player_transform);

    for (entity, collectable, collectable_transform, collectable_size) in collectables.iter() {
        // Get collectable's detection range if allowed.
        let Some(detection_range) = collectable.get_detection_range(range.current() as f32, **collectable_size)
        else {
            continue;
        };

        // Check for collision with the collectable's detection range.
        // - We convert to circle for collectable detection.
        let entity_aabb = AabbSize(detection_range)
            .get_2d(collectable_transform)
            .bounding_circle();
        if !entity_aabb.intersects(&player_aabb) {
            continue;
        }

        // Add attraction.
        c.entity(entity).try_insert(Attraction::new(
            player_entity,
            constants.collectable_max_vel,
            constants.collectable_accel,
            Vec2::ZERO,
            0.,
            false,
        ));
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Item that can be collected by the player.
#[derive(Component, Reflect, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum Collectable
{
    Exp(usize),
    Karma(usize),
    HealthPack,
    //todo: Powerup ???
}

impl Collectable
{
    pub fn get_detection_range(&self, range: f32, _size: Vec2) -> Option<Vec2>
    {
        match self {
            Self::Exp(..) | Self::Karma(..) => Some(Vec2::splat(range)),
            Self::HealthPack => None,
        }
    }

    pub fn spawn(&self, c: &mut Commands, constants: &GameConstants, images: &ImageMap, location: Vec2)
    {
        // Hack: scale up the sprite based on its relative value.
        let (params, texture, scale) = match self {
            Self::Exp(exp) => (
                AabbSize(constants.collectable_exp_size),
                &constants.collectable_exp_texture,
                (*exp as f32).sqrt(),
            ),
            Self::Karma(karma) => (
                AabbSize(constants.collectable_karma_size),
                &constants.collectable_karma_texture,
                (*karma as f32).sqrt(),
            ),
            Self::HealthPack => (
                AabbSize(constants.collectable_healthpack_size),
                &constants.collectable_healthpack_texture,
                1.0,
            ),
        };

        let mut transform = Transform::from_translation(location.extend(0.));
        transform.scale.x = scale.max(1.);
        transform.scale.y = scale.max(1.);

        c.spawn((
            *self,
            params,
            EffectZone::<Player>::new(EffectZoneConfig::SelfDestructSingle, apply_collectable_effect),
            SpatialBundle::from_transform(transform),
            SpriteLayer::Objects,
            StateScoped(GameState::Play),
            images.get(texture),
            Sprite::default(),
            BoundInMap,
        ));
    }
}

impl Default for Collectable
{
    fn default() -> Self
    {
        Self::Exp(0)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Collection of collectables that can be dropped from a unit when it dies.
#[derive(Component, Deref, Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CollectableDrop(SmallVec<[Collectable; 1]>);

//-------------------------------------------------------------------------------------------------------------------

#[derive(SystemSet, Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct CollectablesUpdateSet;

//-------------------------------------------------------------------------------------------------------------------

pub struct CollectablesPlugin;

impl Plugin for CollectablesPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_type::<Collectable>()
            .add_systems(Update, handle_collectable_detection.in_set(CollectablesUpdateSet))
            .add_systems(Update, handle_collectable_drops.in_set(DamageSet::HandleDeaths));
    }
}

//-------------------------------------------------------------------------------------------------------------------
