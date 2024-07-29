use std::f32::consts::TAU;
use std::time::Duration;

use bevy::ecs::world::Command;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn spawn_mobs(
    mut event_count: Local<usize>,
    mut c: Commands,
    clock: Res<GameClock>,
    constants: ReactRes<GameConstants>,
    mob_data: Res<MobDatabase>,
    animations: Res<SpriteAnimations>,
    mut rng: ResMut<GameRng>,
    mut sequence: ResMut<SpawnSequence>,
    mut active_events: ResMut<ActiveEvents>,
    camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    window: Query<&Window, With<PrimaryWindow>>,
    player: Query<(Entity, &Transform), With<Player>>,
    mobs: Query<&InSpawnEvent, With<Mob>>,
)
{
    // Extract events (they are sorted in reverse order).
    loop {
        let Some(last) = sequence.sequence.last() else { break };
        if last.start_time_secs > clock.elapsed_secs() {
            break;
        }
        let last = sequence.sequence.pop().unwrap();
        //tracing::error!("adding event {:?} {:?}", last, clock.elapsed);
        active_events.push(ActiveSpawnEvent::new(*event_count, last, clock.elapsed));
        *event_count += 1;
    }

    if active_events.len() == 0 {
        return;
    }

    // Update current-alive count for all active events.
    for active_event in active_events.iter_mut() {
        active_event.alive_count = 0;
    }
    for in_event in mobs.iter() {
        let Some(event) = active_events.iter_mut().find(|e| e.id == in_event.0) else { continue };
        event.alive_count += 1;
    }

    // Compute spawn radius from window radius and buffer factor.
    let (camera, cam_global) = camera.single();
    let window = window.single();
    let (cam_lower_left, cam_upper_right) = get_camera_corners(&camera, &cam_global, &window);
    let viewport_radius = Vec2 {
        x: (cam_upper_right.x - cam_lower_left.x) / 2.,
        y: (cam_upper_right.y - cam_lower_left.y) / 2.,
    }
    .length();
    let spawn_radius = viewport_radius + constants.spawn_radius_buffer;

    // Add spawns.
    let (player_entity, player_transform) = player.single();
    let rng = rng.rng();

    for ActiveSpawnEvent {
        id,
        event,
        next_spawn_time,
        alive_count,
        total_wave_size,
        total_spawned,
    } in active_events.iter_mut()
    {
        // Check if on a spawn time.
        if clock.elapsed < *next_spawn_time {
            continue;
        }
        *next_spawn_time += Duration::from_secs(event.wave_cooldown_secs);

        // Get num extra mobs needed.
        // - Wait until we've spawned enough mobs before applying the 'spawn extra' logic.
        let extra = if *total_spawned >= event.min_alive {
            event
                .min_alive
                .saturating_sub(*alive_count)
                .saturating_sub(*total_wave_size) as f32
        } else {
            0.
        };

        // Spawn each desired mob.
        for (count, mob_name) in event.mobs_per_wave.iter() {
            // Adjust size proportional to extra needed.
            //tracing::warn!("spawning {} {} extra {}", count, mob_name, extra);
            let mut count = *count;
            if extra > 0. {
                count += (extra * ((count as f32) / (*total_wave_size as f32)))
                    .max(1.)
                    .round() as usize;
            }

            // Spawn randomly around the player.
            let Some(mob_data) = mob_data.get(mob_name) else {
                tracing::error!("failed accessing mob data for {:?}, skipping spawn", mob_name);
                continue;
            };

            // Spawn entities.
            for _ in 0..count {
                // Randomly select radial direction.
                let direction = rng.gen_range((0.)..TAU);

                // Calculate desired spawn location.
                let mut point_transform = Transform::from_translation(Vec3::default().with_x(spawn_radius));
                point_transform.translate_around(Vec3::default(), Quat::from_rotation_z(direction));
                point_transform.translation += player_transform.translation;

                // Select a slightly random location to spawn so all entities don't completely stack on each other.
                let factor = constants.spawn_adjustment_size;
                let adjustment = Vec2 {
                    x: rng.gen_range(-factor..factor),
                    y: rng.gen_range(-factor..factor),
                };
                let mut entity_transform = point_transform;
                entity_transform.translation += adjustment.extend(0.);

                // SPAWN IT
                mob_data.spawn(
                    &mut c,
                    rng,
                    &constants,
                    entity_transform,
                    player_entity,
                    &animations,
                    *id,
                );
            }

            // Count it.
            *total_spawned += count;
        }
    }

    // Clean up empty events.
    active_events.retain(|e| e.event.duration_secs + e.event.start_time_secs > clock.elapsed_secs());
}

//-------------------------------------------------------------------------------------------------------------------

fn insert_spawn_sequence(mut c: Commands, day: ReactRes<Day>, schedule: Res<SpawnSchedule>)
{
    let sch = &schedule.schedule;
    // If we run out of sequences, replay the last day.
    let sequence = sch
        .iter()
        .find(|s| s.day == day.get())
        .or_else(|| sch.get(sch.len() - 1))
        .cloned()
        .unwrap_or_default();
    c.insert_resource(sequence);
    c.insert_resource(ActiveEvents::default());
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Default)]
struct ActiveSpawnEvent
{
    id: usize,
    event: SpawnEvent,
    total_wave_size: usize,

    alive_count: usize,
    next_spawn_time: Duration,
    total_spawned: usize,
}

impl ActiveSpawnEvent
{
    fn new(id: usize, event: SpawnEvent, current_time: Duration) -> Self
    {
        let total_wave_size = event
            .mobs_per_wave
            .iter()
            .map(|(count, _)| *count)
            .reduce(|a, b| a + b)
            .unwrap_or_default();
        Self {
            id,
            event,
            total_wave_size,

            alive_count: 0,
            next_spawn_time: current_time,
            total_spawned: 0,
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Deref, DerefMut, Debug, Default)]
struct ActiveEvents(Vec<ActiveSpawnEvent>);

//-------------------------------------------------------------------------------------------------------------------

/// Id of the spawn event a mob belongs to.
#[derive(Component, Debug)]
pub struct InSpawnEvent(pub usize);

//-------------------------------------------------------------------------------------------------------------------

#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpawnEvent
{
    pub start_time_secs: u64,
    pub duration_secs: u64,
    pub wave_cooldown_secs: u64,
    /// Waits until this many total have spawned before amplified future waves.
    pub min_alive: usize,
    pub mobs_per_wave: Vec<(usize, String)>,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpawnSequence
{
    day: usize,
    sequence: Vec<SpawnEvent>,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpawnSchedule
{
    /// Spawn sequences for each day.
    schedule: Vec<SpawnSequence>,
}

impl Command for SpawnSchedule
{
    fn apply(mut self, w: &mut World)
    {
        self.schedule.sort_unstable_by(|a, b| a.day.cmp(&b.day));
        self.schedule.dedup_by(|a, b| {
            let eq = a.day == b.day;
            if eq {
                tracing::warn!("removing spawn sequence with duplicate day {:?}", b);
            }
            eq
        });

        // Sort in reverse order so removals are cheaper.
        for sequence in &mut self.schedule {
            sequence
                .sequence
                .sort_unstable_by(|a, b| b.start_time_secs.cmp(&a.start_time_secs));
        }

        if let Some(mut schedule) = w.get_resource_mut::<SpawnSchedule>() {
            for sequence in self.schedule.drain(..) {
                let Some(insertion) = schedule.schedule.iter().position(|s| s.day == sequence.day) else {
                    schedule.schedule.push(sequence);
                    continue;
                };
                tracing::warn!("overwriting spawn sequence for day {}", sequence.day);
                schedule.schedule[insertion] = sequence;
            }
        } else {
            w.insert_resource(self);
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct SpawningPlugin;

impl Plugin for SpawningPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_command::<SpawnSchedule>()
            .init_resource::<SpawnSchedule>()
            .add_systems(OnEnter(PlayState::Day), insert_spawn_sequence)
            .add_systems(PreUpdate, spawn_mobs.run_if(in_state(PlayState::Day)));
    }
}

//-------------------------------------------------------------------------------------------------------------------
