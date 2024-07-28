use std::time::Duration;

use bevy::ecs::world::Command;
use bevy::prelude::*;
use bevy_cobweb::react::ReactRes;
use bevy_cobweb_ui::loading::CobwebAssetRegistrationAppExt;
use serde::{Deserialize, Serialize};

use crate::*;

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
            &PlayerDirection,
        ),
        With<Player>,
    >,
    player_powerups: ReactRes<PlayerPowerups>,
    config: Res<CarBatteryConfig>,
)
{
    let Ok((
        player_entity,
        mut ability,
        cdr,
        area_size,
        Transform { translation: Vec3 { x: player_x, y: player_y, .. }, .. },
        p_dir,
    )) = player.get_single_mut()
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

    let player_dir: Dir2 = (*p_dir).into();
    let behind_player_dir = -player_dir;

    // Spawn projectile.
    let damage = config.get_damage(level);
    ProjectileConfig {
        projectile_type: ProjectileType::Pulse {
            damage,
            cooldown_ms: (1000. / config.shock_pulse_frequency) as u64,
            area: config.size,
        },
        velocity_tps: 0.,
        animation: config.animation.clone(),
        size: config.size,
        effect_animation: Some(config.shock_animation.clone()),
        max_lifetime_ms: Some(config.duration_ms),
        sprite_layer: Some(SpriteLayer::GroundEffect),
        ..default()
    }
    .create_projectile::<Mob>(
        &mut c,
        &clock,
        &animations,
        player_entity,
        Vec2 { x: *player_x, y: *player_y } + config.drop_offset * behind_player_dir,
        PlayerDirection::Right.into(),
        &area_size,
    );

    // Update cooldown.
    ability.next_drop_time = time + config.get_cooldown(level, &cdr);
}

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
    /// This is the size of the damage effect zone, not the battery itself.
    pub size: Vec2,
    pub damage_by_level: Vec<usize>,
    pub cooldown_by_level_ms: Vec<u64>,
    /// Offset relative to player from where the battery is dropped on the ground.
    pub drop_offset: f32,
    pub shock_animation: String,
    /// In Hz
    pub shock_pulse_frequency: f32,
    pub duration_ms: u64,
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
            .add_systems(Update, car_battery_placement.in_set(AbilitiesUpdateSet));
    }
}

//-------------------------------------------------------------------------------------------------------------------
