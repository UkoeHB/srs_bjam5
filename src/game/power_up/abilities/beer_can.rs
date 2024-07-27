use std::time::Duration;

use bevy::ecs::world::Command;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use serde::{Deserialize, Serialize};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Checks if the beercan ability needs to be set up on the player.
fn add_beercan_ability(
    mut c: Commands,
    player: Query<(Entity, Has<BeerCanAbility>), With<Player>>,
    player_powerups: ReactRes<PlayerPowerups>,
    config: Res<BeerCanConfig>,
)
{
    let Ok((entity, has_ability)) = player.get_single() else { return };
    if has_ability {
        return;
    }
    if player_powerups.get(&config.name).is_none() {
        return;
    }

    c.entity(entity).try_insert(BeerCanAbility::default());
}

//-------------------------------------------------------------------------------------------------------------------

fn update_beer_can_powerup(
    mut c: Commands,
    clock: Res<GameClock>,
    animations: Res<SpriteAnimations>,
    mut player: Query<(&Transform, &mut BeerCanAbility), With<Player>>,
    mobs: Query<&Transform, (With<Mob>, Without<Player>)>,
    player_powerups: ReactRes<PlayerPowerups>,
    config: Res<BeerCanConfig>,
)
{
    let Ok((transform, mut ability)) = player.get_single_mut() else { return };
    let Some(power_up) = player_powerups.get(&config.name) else { return };

    // Check cooldown.
    let time = clock.elapsed;
    if time < ability.next_fire_time {
        return;
    }

    // Identify nearest mob.
    // TODO: this is not an efficient approach
    let mut nearest: Option<(f32, Vec2)> = None;
    let player_loc = transform.translation.truncate();
    for mob_transform in mobs.iter() {
        let delta = mob_transform.translation.truncate() - player_loc;
        let new_distance = delta.length_squared();
        let Some((distance, _)) = nearest else {
            nearest = Some((new_distance, delta));
            continue;
        };
        if new_distance >= distance {
            continue;
        }
        nearest = Some((new_distance, delta));
    }

    // If no mobs to kill, don't fire.
    let Some((distance_squared, delta)) = nearest else {
        return;
    };
    let nearest_dir = Dir2::new(delta).unwrap_or(Dir2::new_unchecked(Vec2::default().with_x(1.)));

    // If nearest isn't close enough, don't fire.
    if distance_squared.sqrt() > config.detection_range {
        return;
    }

    // Spawn projectile.
    let damage = config.get_damage(power_up.level);
    ProjectileConfig {
        projectile_type: ProjectileType::Explosion { damage, area: config.explosion_size },
        velocity_tps: config.velocity_tps,
        animation: config.animation.clone(),
        size: config.size,
        effect_animation: Some(config.explosion_animation.clone()),
        ..default()
    }
    .create_projectile::<Mob>(
        &mut c,
        &clock,
        &animations,
        player_loc + nearest_dir.rotation_from_x() * config.launch_offset,
        nearest_dir,
    );

    // Update cooldown.
    ability.next_fire_time = time + config.get_cooldown(power_up.level);
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Debug, Default)]
struct BeerCanAbility
{
    next_fire_time: Duration,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Default, Reflect, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct BeerCanConfig
{
    pub name: String,
    pub description: String,
    pub animation: String,
    pub icon: String,
    pub size: Vec2,
    pub damage_by_level: Vec<usize>,
    pub cooldown_by_level_ms: Vec<u64>,
    /// Min distance a mob must be before firing.
    pub detection_range: f32,
    pub velocity_tps: f32,
    /// Offset relative to player from where the projectile should be launched.
    pub launch_offset: Vec2,
    pub explosion_animation: String,
    pub explosion_size: Vec2,
}

impl BeerCanConfig
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

impl Command for BeerCanConfig
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

pub struct BeerCanPlugin;

impl Plugin for BeerCanPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_command::<BeerCanConfig>()
            .init_resource::<BeerCanConfig>()
            .add_systems(PreUpdate, add_beercan_ability.run_if(in_state(PlayState::Day)))
            .add_systems(Update, update_beer_can_powerup.in_set(PowerUpUpdateSet));
    }
}

//-------------------------------------------------------------------------------------------------------------------
