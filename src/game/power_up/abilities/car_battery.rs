use std::time::Duration;

use bevy::ecs::world::Command;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::loading::CobwebAssetRegistrationAppExt;
use serde::{Deserialize, Serialize};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn apply_car_battery_damage_impl(
    In((effect, target)): In<(Entity, Entity)>,
    mut events: EventWriter<DamageEvent>,
    damage: Query<&CarBatteryDamage>,
    player: Query<Entity, With<Player>>,
)
{
    let Ok(damage) = damage.get(effect) else { return };
    let Ok(player_entity) = player.get_single() else { return };
    events.send(DamageEvent { source: player_entity, target, damage: damage.0 });
}

//-------------------------------------------------------------------------------------------------------------------

fn apply_car_battery_damage(effect: Entity, target: Entity, c: &mut Commands)
{
    c.syscall((effect, target), apply_car_battery_damage_impl);
}

//-------------------------------------------------------------------------------------------------------------------

fn apply_car_battery_effect_impl(
    In((projectile, target)): In<(Entity, Entity)>,
    mut c: Commands,
    animations: Res<SpriteAnimations>,
    batteries: Query<(&Transform, &CarBattery)>,
)
{
    let Ok((transform, battery)) = batteries.get(projectile) else { return };
    if battery.target != target {
        return;
    }

    // Clean up attractor and self.
    c.entity(target).despawn_recursive();
    c.entity(projectile).despawn_recursive();

    // Spawn damaging effect.
    c.spawn((
        SpatialBundle::from_transform(*transform), //note: adopts sprite scaling from battery
        StateScoped(GameState::Play),
        DespawnOnAnimationCycle,
        SpriteLayer::Projectiles,
        EffectZone::<Mob>::new(
            EffectZoneConfig::ApplyAndRegen { cooldown_ms: 1_000_000 },
            apply_car_battery_damage,
        ),
        CarBatteryDamage(battery.damage),
        AabbSize(battery.effect_size),
    ))
    .set_sprite_animation(&animations, &battery.animation);
}

//-------------------------------------------------------------------------------------------------------------------

fn apply_car_battery_effect(projectile: Entity, target: Entity, c: &mut Commands)
{
    c.syscall((projectile, target), apply_car_battery_effect_impl);
}

//-------------------------------------------------------------------------------------------------------------------

/// Checks if the ability needs to be set up on the player.
fn add_car_battery_ability(
    mut c: Commands,
    player: Query<(Entity, Has<CarBatteryAbility>), With<Player>>,
    player_powerups: ReactRes<PlayerPowerups>,
    config: Res<CarBatteryConfig>,
)
{
    let Ok((entity, has_ability)) = player.get_single() else { return };
    if has_ability {
        return;
    }
    if player_powerups.get(&config.name) == 0 {
        return;
    }

    c.entity(entity).try_insert(CarBatteryAbility::default());
}

//-------------------------------------------------------------------------------------------------------------------

fn car_battery_placement(
    mut c: Commands,
    clock: Res<GameClock>,
    animations: Res<SpriteAnimations>,
    mut player: Query<
        (
            Entity,
            &mut CarBatteryAbility,
            &CooldownReduction,
            &AreaSize,
            &Transform,
        ),
        With<Player>,
    >,
    player_powerups: ReactRes<PlayerPowerups>,
    config: Res<CarBatteryConfig>,
    mobs: Query<(&Transform, &Health), With<Mob>>,
)
{
    let Ok((player_entity, mut ability, cdr, area_size, Transform { translation, .. })) = player.get_single_mut()
    else {
        return;
    };
    let level = player_powerups.get(&config.name);
    if level == 0 {
        return;
    }

    let time = clock.elapsed;
    if time < ability.next_drop_time {
        return;
    }

    // Find highest-health and closest enemy in range.
    // (health, distance squared, location)
    let player_loc = translation.truncate();
    let range_squared = config.throw_range * config.throw_range;
    let mut best: (usize, f32, Vec2) = (0, 0., player_loc);
    for (transform, health) in mobs.iter() {
        if health.current() < best.0 {
            continue;
        }

        let loc = transform.translation.truncate();
        let distance_squared = (loc - player_loc).length_squared();
        if distance_squared > range_squared {
            continue;
        }

        if health.current() > best.0 {
            best = (health.current(), distance_squared, loc);
            continue;
        }

        if distance_squared >= best.1 {
            continue;
        }

        best = (health.current(), distance_squared, loc);
    }

    // Don't do anything if no mobs found.
    if best.0 == 0 {
        return;
    }
    let target_dir = Dir2::new(best.2 - player_loc).unwrap_or(Dir2::new_unchecked(Vec2::default().with_x(1.)));

    // Spawn attractor entity.
    let attractor = c
        .spawn((
            SpatialBundle::from_transform(Transform::from_translation(best.2.extend(0.))),
            AttractionSource::LowPriority,
            BatteryAttractor,
            AabbSize(Vec2::splat(1.)),
        ))
        .id();

    // Spawn battery entity, attracted to attractor.
    // - When battery effect hits the attractor entity, despawn it and the battery and spawn the battery's
    //   electrocution effect.
    let projectile = ProjectileConfig {
        projectile_type: ProjectileType::Continuous { damage: 0, cooldown_ms: 1_000_000 },
        velocity_tps: 0.,
        animation: config.animation.clone(),
        size: config.size,
        max_lifetime_ms: Some(10_000),
        sprite_layer: Some(SpriteLayer::Projectiles),
        ..default()
    }
    .create_projectile::<BatteryAttractor>(
        &mut c,
        &clock,
        &animations,
        player_entity,
        player_loc + config.release_offset * target_dir,
        PlayerDirection::Right.into(),
        &area_size,
        Some(apply_car_battery_effect),
    )
    .unwrap();
    c.entity(projectile).insert((
        CarBattery {
            target: attractor,
            animation: config.shock_animation.clone(),
            damage: config.get_damage(level),
            effect_size: area_size.calculate_area(config.damage_size),
        },
        Attraction::new(attractor, config.velocity_tps, 0., Vec2::default(), 0., false),
    ));

    // Update cooldown.
    ability.next_drop_time = time + config.get_cooldown(level, &cdr);
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component)]
struct BatteryAttractor;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component)]
struct CarBattery
{
    target: Entity,
    animation: String,
    damage: usize,
    effect_size: Vec2,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component)]
struct CarBatteryDamage(usize);

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Debug, Default)]
struct CarBatteryAbility
{
    next_drop_time: Duration,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Default, Reflect, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CarBatteryConfig
{
    pub name: String,
    pub description: String,
    pub animation: String,
    pub icon: String,
    /// Size of battery.
    pub size: Vec2,
    pub damage_by_level: Vec<usize>,
    pub cooldown_by_level_ms: Vec<u64>,
    pub shock_animation: String,

    /// This is the size of the damage effect zone.
    pub damage_size: Vec2,

    /// Offset relative to player from where the battery is thrown.
    pub release_offset: f32,
    pub throw_range: f32,
    pub velocity_tps: f32,
}

impl CarBatteryConfig
{
    fn get_damage(&self, level: usize) -> usize
    {
        let level = (level.saturating_sub(1)).min(self.damage_by_level.len().saturating_sub(1));
        self.damage_by_level.get(level).cloned().unwrap_or_default()
    }

    fn get_cooldown(&self, level: usize, cdr: &CooldownReduction) -> Duration
    {
        let level = (level.saturating_sub(1)).min(self.cooldown_by_level_ms.len().saturating_sub(1));
        let cooldown = self
            .cooldown_by_level_ms
            .get(level)
            .cloned()
            .unwrap_or_default();

        // Apply cdr.
        let cooldown = cdr.calculate_cooldown(cooldown);

        Duration::from_millis(cooldown)
    }
}

impl Command for CarBatteryConfig
{
    fn apply(self, w: &mut World)
    {
        w.resource_mut::<PowerupBank>().register(PowerupInfo {
            name: self.name.clone(),
            description: self.description.clone(),
            icon: self.icon.clone(),
            ability_type: AbilityType::Active,
        });
        w.insert_resource(self);
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct CarBatteryPlugin;

impl Plugin for CarBatteryPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_command::<CarBatteryConfig>()
            .init_resource::<CarBatteryConfig>()
            .add_systems(PreUpdate, add_car_battery_ability.run_if(in_state(PlayState::Day)))
            .add_systems(Update, car_battery_placement.in_set(AbilitiesUpdateSet))
            .add_effect_target::<BatteryAttractor>();
    }
}

//-------------------------------------------------------------------------------------------------------------------
