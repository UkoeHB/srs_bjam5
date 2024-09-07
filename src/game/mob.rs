use std::collections::HashMap;
use std::time::Duration;

use bevy::ecs::system::EntityCommands;
use bevy::ecs::world::Command;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn update_emitter_mobs(
    mut c: Commands,
    clock: Res<GameClock>,
    animations: Res<SpriteAnimations>,
    player: Query<&Transform, With<Player>>,
    mut emitters: Query<(Entity, &mut Emitter, &Transform, &Attraction), (With<Mob>, Without<Player>)>,
)
{
    let Ok(player_transform) = player.get_single() else { return };
    let time = clock.elapsed;

    for (entity, mut emitter, transform, attraction) in emitters.iter_mut() {
        // Wait for emitters to stop moving.
        if !attraction.is_stopped() {
            continue;
        }

        // Update emitter cooldown.
        if !emitter.update_cooldown(time) {
            continue;
        }

        // Make a new projectile targeting the player.
        emitter.config().create_projectile::<Player>(
            &mut c,
            &clock,
            &animations,
            entity,
            transform.translation.truncate(),
            Dir2::new((player_transform.translation - transform.translation).truncate())
                .unwrap_or(Dir2::new_unchecked(Vec2::default().with_x(1.))),
            &AreaSize::new(1.0),
            None,
        );
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// On death, try to apply damage to the player. We do this manually so the damage is applied
/// immediately instead of e.g. indirecting through a spawned exploder projectile.
fn handle_exploder_deaths(
    mut c: Commands,
    mut deaths: EventReader<EntityDeath>,
    mut dmg_events: EventWriter<DamageEvent>,
    animations: Res<SpriteAnimations>,
    exploders: Query<(Entity, &Exploder)>,
    player: Query<(Entity, &Transform), With<Player>>,
    mobs: Query<&Transform, Without<Player>>,
)
{
    for (mob_entity, exploder) in deaths
        .read()
        .filter_map(|death| exploders.get(**death).ok())
    {
        let Exploder { base_damage, base_range, explosion_animation } = exploder.clone();

        // Spawn explosion effect.
        let Ok(mob_transform) = mobs.get(mob_entity) else { continue };
        c.spawn((
            DespawnOnAnimationCycle,
            SpatialBundle::from_transform(*mob_transform),
            SpriteLayer::Objects,
            StateScoped(GameState::Play),
        ))
        .set_sprite_animation(&animations, explosion_animation);

        // Check if player is in range.
        let Ok((player, player_transform)) = player.get_single() else { continue };
        let distance = (player_transform.translation - mob_transform.translation).length();
        if distance > base_range {
            continue;
        }

        // Send damage event.
        dmg_events.send(DamageEvent { source: mob_entity, target: player, damage: base_damage });
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn apply_collider_effect_impl(
    In((source, target)): In<(Entity, Entity)>,
    mut events: EventWriter<DamageEvent>,
    colliders: Query<&Collider>,
)
{
    let Ok(collider) = colliders.get(source) else { return };
    events.send(DamageEvent { source, target, damage: collider.damage });
}

pub fn apply_collider_effect(collider: Entity, target: Entity, c: &mut Commands)
{
    c.syscall((collider, target), apply_collider_effect_impl);
}

/// Component for collider effects.
#[derive(Component, Debug)]
pub struct Collider
{
    pub damage: usize,
}

//-------------------------------------------------------------------------------------------------------------------

/// Component for emitter mobs.
#[derive(Component, Debug)]
pub struct Emitter
{
    cooldown_ms: u64,
    projectile: ProjectileConfig,

    next_fire_time: Option<Duration>,
}

impl Emitter
{
    pub fn new(cooldown_ms: u64, projectile: ProjectileConfig) -> Self
    {
        Self { cooldown_ms, projectile, next_fire_time: None }
    }

    pub fn config(&self) -> &ProjectileConfig
    {
        &self.projectile
    }

    /// Returns true if the emitter should fire.
    pub fn update_cooldown(&mut self, time: Duration) -> bool
    {
        if let Some(next) = self.next_fire_time {
            if time < next {
                return false;
            }
        }

        self.next_fire_time = Some(time + Duration::from_millis(self.cooldown_ms));
        true
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Component for mobs that explode on death.
#[derive(Component, Clone, Debug)]
pub struct Exploder
{
    base_damage: usize,
    base_range: f32,
    explosion_animation: String,
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
        base_cooldown_millis: u64,
        /// Range in transform units.
        base_fire_range: f32,
        projectile: ProjectileConfig,
    },
    OnDeath(MobOnDeathType),
    //todo: Spawner can spawn different mobs with an internal cooldown per mob type
}

impl MobType
{
    //todo: can 'amplify' the mob stats here
    /// Returns the distance from the player where the entity should stop being attracted.
    pub fn setup_in_entity(&self, constants: &GameConstants, ec: &mut EntityCommands, start_pos: Vec2) -> f32
    {
        match self.clone() {
            Self::Collider { base_damage, base_cooldown_millis } => {
                ec.insert((
                    EffectZone::<Player>::new(
                        EffectZoneConfig::ApplyAndRegen { cooldown_ms: base_cooldown_millis },
                        apply_collider_effect,
                    ),
                    PrevLocation(start_pos),
                    Collider { damage: base_damage },
                ));
                constants.collider_mob_stop_distance
            }
            Self::Emitter { base_cooldown_millis, base_fire_range, projectile } => {
                ec.insert(Emitter::new(base_cooldown_millis, projectile));
                base_fire_range
            }
            Self::OnDeath(on_death) => match on_death {
                MobOnDeathType::Explode { base_damage, base_range, explosion_animation } => {
                    ec.insert(Exploder { base_damage, base_range, explosion_animation });
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
    /// [ (probability of drop, drop) ]
    pub drops: Vec<(f32, CollectableDrop)>,
    pub auto_flip_sprite: bool,
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
        event_id: usize,
    )
    {
        let offset = constants.mob_attraction_offset;
        let target_offset =
            Vec2::new(rng.gen_range(-offset..=offset), rng.gen_range(-offset..=offset)).clamp_length_max(offset);

        let mut ec = c.spawn_empty();
        let stop_distance =
            self.mob_type
                .setup_in_entity(constants, &mut ec, entity_transform.translation.truncate());
        ec.insert((
            Mob,
            SpatialBundle::from_transform(entity_transform),
            SpriteLayer::Objects,
            AabbSize(self.hitbox),
            Health::new(self.base_health),
            Armor::new(self.base_armor),
            Attraction::new(
                player_entity,
                self.base_speed_tps,
                0.,
                target_offset,
                stop_distance,
                self.auto_flip_sprite,
            ),
            DespawnOnDeath,
            StateScoped(GameState::Play),
            BoundInMap,
            InSpawnEvent(event_id),
        ))
        .set_sprite_animation(&animations, &self.animation);

        if let Some(drop) = self.select_collectable_drop(rng) {
            ec.insert(drop);
        }
    }

    fn select_collectable_drop(&self, rng: &mut ChaCha8Rng) -> Option<CollectableDrop>
    {
        let selection = rng.gen_range((0.)..(1.));
        let mut accumulated = 0.;
        for (probability, drop) in self.drops.iter() {
            accumulated += probability;
            if accumulated < selection {
                continue;
            }
            return Some(drop.clone());
        }
        None
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

#[derive(SystemSet, Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct MobUpdateSet;

//-------------------------------------------------------------------------------------------------------------------

pub struct MobPlugin;

impl Plugin for MobPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_command::<MobDatabase>()
            .init_resource::<MobDatabase>()
            .add_systems(Update, update_emitter_mobs.in_set(MobUpdateSet))
            .add_systems(Update, handle_exploder_deaths.in_set(DamageSet::HandleDeaths));
    }
}

//-------------------------------------------------------------------------------------------------------------------
