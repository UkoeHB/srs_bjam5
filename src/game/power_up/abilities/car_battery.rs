use std::time::Duration;

use bevy::ecs::world::Command;
use bevy::prelude::*;
use bevy_cobweb::react::ReactRes;
use bevy_cobweb_ui::loading::CobwebAssetRegistrationAppExt;
use serde::{Deserialize, Serialize};

use crate::*;

// TODO:
// Give player the car battery ability

fn car_battery_placement(
    mut c: Commands,
    clock: Res<GameClock>,
    animations: Res<SpriteAnimations>,
    mut player: Query<(&mut CarBatteryAbility, &Transform, &PrevLocation), With<Player>>,
    player_powerups: ReactRes<PlayerPowerups>,
    config: Res<CarBatteryConfig>,
)
{
    let Ok((
        mut ability,
        Transform { translation: Vec3 { x: player_x, y: player_y, .. }, .. },
        PrevLocation(prev_loc),
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

    let Ok(dir_to_prev) = Dir2::new(*prev_loc - Vec2 { x: *player_x, y: *player_y }) else { return };

    // Spawn projectile.
    let damage = config.get_damage(level);
    ProjectileConfig {
        projectile_type: ProjectileType::Continuous {
            damage,
            cooldown_ms: (1000. / config.shock_pulse_frequency) as u64,
        },
        velocity_tps: 0.,
        animation: config.animation.clone(),
        size: config.size,
        effect_animation: Some(config.shock_animation.clone()),
        ..default()
    }
    .create_projectile::<Mob>(
        &mut c,
        &clock,
        &animations,
        config.drop_offset * dir_to_prev,
        dir_to_prev,
    );

    // Update cooldown.
    ability.next_drop_time = time + config.get_cooldown(level);
}

#[derive(Component, Debug, Default)]
struct CarBatteryAbility
{
    next_drop_time: Duration,
}

#[derive(Resource, Default, Reflect, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CarBatteryConfig
{
    pub name: String,
    pub description: String,
    pub animation: String,
    pub size: Vec2,
    pub damage_by_level: Vec<usize>,
    pub cooldown_by_level_ms: Vec<u64>,
    /// Offset relative to player from where the battery is dropped on the ground.
    pub drop_offset: f32,
    pub shock_animation: String,
    /// In Hz
    pub shock_pulse_frequency: f32,
}

impl CarBatteryConfig
{
    fn get_damage(&self, level: usize) -> usize
    {
        let level = (level.saturating_sub(1)).min(self.damage_by_level.len().saturating_sub(1));
        self.damage_by_level.get(level).cloned().unwrap_or_default()
    }

    fn get_cooldown(&self, level: usize) -> Duration
    {
        let level = (level.saturating_sub(1)).min(self.cooldown_by_level_ms.len().saturating_sub(1));
        self.cooldown_by_level_ms
            .get(level)
            .map(|cd| Duration::from_millis(*cd))
            .unwrap_or_default()
    }
}

impl Command for CarBatteryConfig
{
    fn apply(self, w: &mut World)
    {
        w.resource_mut::<PowerupBank>().register(PowerupInfo {
            ability_type: AbilityType::Active,
            name: self.name.clone(),
            description: self.description.clone(),
            icon: self.animation.clone(),
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
            .add_systems(Update, car_battery_placement);
    }
}
