use std::f32::consts::TAU;

use bevy::ecs::world::Command;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

const MIN_RAMP_UP: f32 = 0.05;

fn calc_magnitude(progress: f32, ramp_up_fraction: f32) -> f32
{
    (progress / ramp_up_fraction.max(MIN_RAMP_UP)).min(1.)
}

//-------------------------------------------------------------------------------------------------------------------

fn spawn_mobs(
    mut last_tracked_day: Local<usize>,
    mut active_events: Local<Vec<ActiveSpawnEvent>>,
    mut c: Commands,
    day: ReactRes<Day>,
    clock: Res<GameClock>,
    constants: ReactRes<GameConstants>,
    mob_data: Res<MobDatabase>,
    animations: Res<SpriteAnimations>,
    mut rng: ResMut<GameRng>,
    mut sequence: ResMut<SpawnSequence>,
    camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    window: Query<&Window, With<PrimaryWindow>>,
    player: Query<(Entity, &Transform), With<Player>>,
)
{
    // Cleanup for day changeover
    if *last_tracked_day != day.get() {
        active_events.clear();
        *last_tracked_day = day.get();
    }

    // Extract events (they are sorted in reverse order).
    loop {
        let Some(last) = sequence.sequence.last() else { break };
        if last.start_time_secs < clock.elapsed_secs() {
            break;
        }
        let last = sequence.sequence.pop().unwrap();
        active_events.push(ActiveSpawnEvent::new(last, constants.spawn_point_cadence_secs));
    }

    if active_events.len() == 0 {
        return;
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

    for ActiveSpawnEvent { event, spawn_point_size, spawned_quantity, next_spawn_point } in
        active_events.iter_mut()
    {
        // Time since event started.
        let duration = clock.elapsed_secs().saturating_sub(event.start_time_secs);

        // Check if on a spawn time.
        let cadence = constants.spawn_point_cadence_secs.max(1);
        if duration as usize % cadence != 0 {
            continue;
        }

        // Check if on a new spawn time.
        let spawn_point = duration as usize / cadence;
        if spawn_point < *next_spawn_point {
            continue;
        }
        *next_spawn_point = spawn_point + 1;

        // Calculate spawn magnitude.
        let progress = ((duration as f32) / (event.duration_secs as f32).max(1.)).min(1.);
        let magnitude = calc_magnitude(progress, event.ramp_up_fraction);

        // Calculate number of mobs to spawn right now.
        let mut spawn_point_remaining = ((*spawn_point_size as f32) * magnitude).round() as usize;
        spawn_point_remaining = spawn_point_remaining.min(event.total_quantity.saturating_sub(*spawned_quantity));
        let mut clump_size = ((event.max_clump_size as f32) * magnitude).round() as usize;

        // Update the spawned quantity.
        *spawned_quantity += spawn_point_remaining;

        // Spawn clumps randomly around the player.
        let Some(mob_data) = mob_data.get(&event.mob_name) else {
            *spawned_quantity = event.total_quantity;
            tracing::error!("failed accessing mob data for {:?}, discarding spawn event", event.mob_name);
            continue;
        };

        while spawn_point_remaining > 0 {
            // Extract clump.
            clump_size = clump_size.max(1).min(spawn_point_remaining);
            spawn_point_remaining -= clump_size;

            // Randomly select radial direction.
            let direction = rng.gen_range((0.)..TAU);

            // Calculate desired spawn location.
            let mut point_transform = Transform::from_translation(Vec3::default().with_x(spawn_radius));
            point_transform.translate_around(Vec3::default(), Quat::from_rotation_z(direction));
            point_transform.translation += player_transform.translation;

            // Spawn entities in the clump.
            for _ in 0..clump_size {
                // Select a slightly random location to spawn so all entities don't completely stack on each other.
                let factor = constants.spawn_adjustment_size;
                let adjustment = Vec2 {
                    x: rng.gen_range(-factor..factor),
                    y: rng.gen_range(-factor..factor),
                };
                let mut entity_transform = point_transform;
                entity_transform.translation += adjustment.extend(0.);

                // Correct so the entity stays inside the map.
                // - Incorporates mob hit box.
                // - Consider not just truncating the offset from player but also re-rotating so not too many mobs
                //   spawn on top of you when adjacent to a wall?
                // TODO

                // SPAWN IT
                mob_data.spawn(&mut c, rng, &constants, entity_transform, player_entity, &animations);
            }
        }
    }

    // Clean up empty events.
    active_events.retain(|e| e.spawned_quantity < e.event.total_quantity);
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
}

//-------------------------------------------------------------------------------------------------------------------

struct ActiveSpawnEvent
{
    event: SpawnEvent,
    /// Calculated number of mobs to spawn at each cadence point (modified by magnitude).
    spawn_point_size: usize,
    spawned_quantity: usize,
    /// Tracks the next spawn cadence for this event to avoid multi-spawning on a cadence point.
    next_spawn_point: usize,
}

impl ActiveSpawnEvent
{
    fn new(event: SpawnEvent, cadence: usize) -> Self
    {
        // Calculate spawn point size by integrating over magnitudes and dividing into the total.
        let total_points = (event.duration_secs as usize / cadence.max(1))
            + (event.duration_secs as usize % cadence.max(1)).min(1);
        let mut total_magnitude = 0.;
        for n in 0..total_points {
            let progress = (n as f32) / (total_points.saturating_sub(1) as f32).max(1.);
            total_magnitude += calc_magnitude(progress, event.ramp_up_fraction);
        }
        let spawn_point_size = ((event.total_quantity as f32) / total_magnitude.max(1.))
            .round()
            .max(1.) as usize;

        Self {
            event,
            spawn_point_size,
            spawned_quantity: 0,
            next_spawn_point: 0,
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpawnEvent
{
    pub mob_name: String,
    pub start_time_secs: u64,
    pub duration_secs: u64,
    /// Fraction of duration spent ramping up spawn rate before plateau.
    pub ramp_up_fraction: f32,
    pub total_quantity: usize,
    /// SpawnPoint quantity when fully ramped-up.
    pub max_clump_size: usize,
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
