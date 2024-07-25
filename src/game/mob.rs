use std::collections::HashMap;

use bevy::ecs::system::EntityCommands;
use bevy::ecs::world::Command;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::*;

//todo: emitter types should fire on cooldown when not attracted to player

//-------------------------------------------------------------------------------------------------------------------

fn apply_collider_effect_impl(
    In((collider, target)): In<(Entity, Entity)>,
    mut events: EventWriter<DamageEvent>,
    colliders: Query<&Collider>,
)
{
    let Ok(collider) = colliders.get(collider) else { return };
    events.send(DamageEvent { target, damage: collider.damage });
}

//-------------------------------------------------------------------------------------------------------------------

fn apply_collider_effect(collider: Entity, target: Entity, c: &mut Commands)
{
    c.syscall((collider, target), apply_collider_effect_impl);
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Reflect, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MobOnDeathType
{
    Explode
    {
        base_damage: usize,
        base_range: f32,
        /// The animation to display when exploding. The explosion entity auto-despawns when the animation ends.
        explosion_animation: String,
    },
}

impl Default for MobOnDeathType
{
    fn default() -> Self
    {
        Self::Explode {
            base_damage: 0,
            base_range: 0.,
            explosion_animation: "".into(),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Reflect, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MobType
{
    Collider
    {
        base_damage: usize,
        base_cooldown_millis: u64,
    },
    Emitter
    {
        base_damage: usize,
        base_cooldown_millis: u64,
        /// Range in transform units.
        base_fire_range: f32,
    },
    OnDeath(MobOnDeathType),
}

impl MobType
{
    //todo: can 'amplify' the mob stats here
    /// Returns the distance from the player where the entity should stop being attracted.
    pub fn setup_in_entity(&self, constants: &GameConstants, ec: &mut EntityCommands) -> f32
    {
        match self {
            Self::Collider { base_damage, base_cooldown_millis } => {
                ec.insert((
                    EffectZone::<Player>::new(
                        EffectZoneConfig::ApplyAndRegen { cooldown_ms: *base_cooldown_millis as usize },
                        apply_collider_effect,
                    ),
                    Collider { damage: *base_damage },
                ));
                constants.collider_mob_stop_distance
            }
            Self::Emitter { base_damage, base_cooldown_millis, base_fire_range } => {
                ec.insert(Emitter {
                    damage: *base_damage,
                    cooldown: *base_cooldown_millis as usize,
                });
                *base_fire_range
            }
            Self::OnDeath(on_death) => match on_death.clone() {
                MobOnDeathType::Explode { base_damage, base_range, explosion_animation } => {
                    // On death, try to apply damage to the player. We do this manually instead of with an
                    // effect zone so the damage zone can be circular.
                    let mob_entity = ec.id();
                    ec.observe(
                        move |_: Trigger<EntityDeath>,
                              mut c: Commands,
                              mut dmg_events: EventWriter<DamageEvent>,
                              animations: Res<SpriteAnimations>,
                              player: Query<(Entity, &Transform), With<Player>>,
                              mobs: Query<&Transform, Without<Player>>| {
                            let Ok((player, player_transform)) = player.get_single() else { return };
                            let Ok(mob_transform) = mobs.get(mob_entity) else { return };

                            // Check if player is in range.
                            let distance = (player_transform.translation - mob_transform.translation).length();
                            if distance > base_range {
                                return;
                            }

                            // Send damage event.
                            dmg_events.send(DamageEvent { target: player, damage: base_damage });

                            // Spawn explosion effect.
                            c.spawn(DespawnOnAnimationCycle)
                                .set_sprite_animation(&animations, explosion_animation.clone());
                        },
                    );
                    0.
                }
            },
        }
    }
}

impl Default for MobType
{
    fn default() -> Self
    {
        Self::Collider { base_damage: 0, base_cooldown_millis: 1000 }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MobData
{
    pub animation: String,
    /// This should usually equal the animation size, but doesn't have to.
    pub hitbox: Vec2,
    pub base_health: usize,
    pub base_armor: usize,
    pub base_speed_tps: f32,
    pub mob_type: MobType,
}

impl MobData
{
    pub fn spawn(
        &self,
        c: &mut Commands,
        rng: &mut ChaCha8Rng,
        constants: &GameConstants,
        entity_transform: Transform,
        player_entity: Entity,
        animations: &SpriteAnimations,
    )
    {
        let offset = constants.mob_attraction_offset;
        let target_offset =
            Vec2::new(rng.gen_range(-offset..=offset), rng.gen_range(-offset..=offset)).clamp_length_max(offset);

        let mut ec = c.spawn_empty();
        let stop_distance = self.mob_type.setup_in_entity(constants, &mut ec);
        ec.insert((
            Mob,
            SpatialBundle::from_transform(entity_transform),
            SpriteLayer::Objects,
            AabbSize(self.hitbox),
            Health::from_max(self.base_health),
            Armor::new(self.base_armor),
            Attraction::new(player_entity, self.base_speed_tps, 0., target_offset, stop_distance),
            StateScoped(GameState::Play),
        ))
        .set_sprite_animation(&animations, &self.animation);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// [mob name, mob data]
#[derive(Resource, Deref, Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MobDatabase(HashMap<String, MobData>);

impl Command for MobDatabase
{
    fn apply(self, w: &mut World)
    {
        w.insert_resource(self);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Marker component for mob entities.
#[derive(Component, Debug)]
pub struct Mob;

//-------------------------------------------------------------------------------------------------------------------

pub struct MobPlugin;

impl Plugin for MobPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_command::<MobDatabase>()
            .init_resource::<MobDatabase>();
    }
}

//-------------------------------------------------------------------------------------------------------------------
