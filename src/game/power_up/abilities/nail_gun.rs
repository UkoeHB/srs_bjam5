use std::time::Duration;

use bevy::ecs::world::Command;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use serde::{Deserialize, Serialize};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Checks if the nailgun ability needs to be set up on the player.
fn add_nailgun_ability(
    mut c: Commands,
    player: Query<(Entity, Has<NailGunAbility>), With<Player>>,
    player_powerups: ReactRes<PlayerPowerups>,
    config: Res<NailGunConfig>,
)
{
    let Ok((entity, has_ability)) = player.get_single() else { return };
    if has_ability {
        return;
    }
    if player_powerups.get(&config.name) == 0 {
        return;
    }

    c.entity(entity).try_insert(NailGunAbility::default());
}

//-------------------------------------------------------------------------------------------------------------------

fn update_nail_gun_powerup(
    mut c: Commands,
    clock: Res<GameClock>,
    animations: Res<SpriteAnimations>,
    mut player: Query<(Entity, &Transform, &CooldownReduction, &AreaSize, &mut NailGunAbility), With<Player>>,
    player_powerups: ReactRes<PlayerPowerups>,
    config: Res<NailGunConfig>,
)
{
    let Ok((player_entity, transform, cdr, area_size, mut ability)) = player.get_single_mut() else { return };
    let level = player_powerups.get(&config.name);
    if level == 0 {
        return;
    }

    // Check cooldown.
    let time = clock.elapsed;
    if time < ability.next_fire_time {
        return;
    }

    // Spawn projectiles.
    let damage = config.get_damage(level);
    let projectile = ProjectileConfig {
        projectile_type: ProjectileType::SingleUse { damage },
        velocity_tps: config.velocity_tps,
        animation: config.animation.clone(),
        size: config.size,
        ..default()
    };

    let player_loc = transform.translation.truncate();
    for dir in [
        Into::<Dir2>::into(PlayerDirection::Up),
        PlayerDirection::Down.into(),
        PlayerDirection::Left.into(),
        PlayerDirection::Right.into(),
    ] {
        projectile.create_projectile::<Mob>(
            &mut c,
            &clock,
            &animations,
            player_entity,
            player_loc + dir.rotation_from_x() * config.launch_offset,
            dir,
            &area_size,
        );
    }

    // Update cooldown.
    ability.next_fire_time = time + config.get_cooldown(level, &cdr);
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Debug, Default)]
struct NailGunAbility
{
    next_fire_time: Duration,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Default, Reflect, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct NailGunConfig
{
    pub name: String,
    pub description: String,
    pub animation: String,
    pub icon: String,
    pub size: Vec2,
    pub damage_by_level: Vec<usize>,
    pub cooldown_by_level_ms: Vec<u64>,
    pub velocity_tps: f32,
    /// Offset relative to player from where the projectile should be launched.
    pub launch_offset: Vec2,
}

impl NailGunConfig
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

impl Command for NailGunConfig
{
    fn apply(self, w: &mut World)
    {
        w.resource_mut::<PowerupBank>().register(PowerupInfo {
            ability_type: AbilityType::Active,
            name: self.name.clone(),
            description: self.description.clone(),
            icon: self.icon.clone(),
        });
        w.insert_resource(self);
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct NailGunPlugin;

impl Plugin for NailGunPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_command::<NailGunConfig>()
            .init_resource::<NailGunConfig>()
            .add_systems(PreUpdate, add_nailgun_ability.run_if(in_state(PlayState::Day)))
            .add_systems(Update, update_nail_gun_powerup.in_set(AbilitiesUpdateSet));
    }
}

//-------------------------------------------------------------------------------------------------------------------
