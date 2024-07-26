use bevy::math::bounding::IntersectsVolume;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;

use crate::*;

//todo: add observer to OnRemove::<Mob> for spawning drops

//-------------------------------------------------------------------------------------------------------------------

fn apply_collectable_effect_impl(
    In((collectable, _player_entity)): In<(Entity, Entity)>,
    collectables: Query<&Collectable>,
    mut c: Commands,
    mut player: Query<(&mut Level, &mut Health), With<Player>>,
    mut karma: ReactResMut<Karma>,
    mut powerups: ResMut<BufferedPowerUps>,
)
{
    let Ok((mut level, mut health)) = player.get_single_mut() else { return };
    let Ok(collectable) = collectables.get(collectable) else { return };

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
}

fn apply_collectable_effect(source: Entity, target: Entity, c: &mut Commands)
{
    c.syscall((source, target), apply_collectable_effect_impl);
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
    let player_aabb = player_size.get_2d(player_transform);

    for (entity, collectable, collectable_transform, collectable_size) in collectables.iter() {
        // Get collectable's detection range if allowed.
        let Some(detection_range) = collectable.get_detection_range(&constants, **collectable_size) else {
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
        ));
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Item that can be collected by the player.
#[derive(Component, Copy, Clone, Debug)]
pub enum Collectable
{
    Exp(usize),
    Karma(usize),
    HealthPack(usize),
}

impl Collectable
{
    pub fn get_detection_range(&self, constants: &GameConstants, _size: Vec2) -> Option<Vec2>
    {
        match self {
            Self::Exp(..) | Self::Karma(..) => Some(constants.hoover_detection_range),
            Self::HealthPack(..) => None,
        }
    }

    pub fn spawn(&self, c: &mut Commands, constants: &GameConstants, images: &ImageMap, location: Vec2)
    {
        let (params, texture) = match self {
            Self::Exp(..) => (
                AabbSize(constants.collectable_exp_size),
                &constants.collectable_exp_texture,
            ),
            Self::Karma(..) => (
                AabbSize(constants.collectable_karma_size),
                &constants.collectable_karma_texture,
            ),
            Self::HealthPack(..) => (
                AabbSize(constants.collectable_healthpack_size),
                &constants.collectable_healthpack_texture,
            ),
        };

        c.spawn((
            *self,
            params,
            EffectZone::<Player>::new(EffectZoneConfig::SelfDestructSingle, apply_collectable_effect),
            SpatialBundle::from_transform(Transform::from_translation(location.extend(0.))),
            SpriteLayer::Objects,
            StateScoped(GameState::Play),
            images.get(texture),
            Sprite::default(),
        ));
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(SystemSet, Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct CollectablesUpdateSet;

//-------------------------------------------------------------------------------------------------------------------

pub struct CollectablesPlugin;

impl Plugin for CollectablesPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_systems(Update, handle_collectable_detection.in_set(CollectablesUpdateSet));
    }
}

//-------------------------------------------------------------------------------------------------------------------
