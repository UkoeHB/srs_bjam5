use bevy::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn handle_damage_events(
    mut events: EventReader<DamageEvent>,
    mut deaths: EventWriter<EntityDeath>,
    sources: Query<&DamageAmp>,
    mut targets: Query<(&mut Health, &Armor)>,
)
{
    for DamageEvent { source, target, damage } in events.read() {
        let Ok((mut hp, armor)) = targets.get_mut(*target) else { continue };

        // Check if entity is already dead.
        if hp.current() == 0 {
            continue;
        }

        // Calculate damage to apply.
        let damage = sources
            .get(*source)
            .map(|a| a.calculate_damage(*damage as f32))
            .unwrap_or(*damage as f32);
        let damage = armor.calculate_damage(damage);
        hp.remove(damage.round() as usize);

        // Check for entity death.
        if hp.current() == 0 {
            deaths.send(EntityDeath(*target));
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn despawn_dead_entities(
    mut c: Commands,
    mut events: EventReader<EntityDeath>,
    should_despawn: Query<(), With<DespawnOnDeath>>,
)
{
    for EntityDeath(entity) in events.read() {
        if !should_despawn.contains(*entity) {
            continue;
        }
        c.entity(*entity).despawn_recursive();
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Marker components for entities that should despawn when receiving `EntityDeath` events.
#[derive(Component, Debug)]
pub struct DespawnOnDeath;

//-------------------------------------------------------------------------------------------------------------------

/// Event sent to apply damage to an entity.
#[derive(Event, Debug, Copy, Clone)]
pub struct DamageEvent
{
    pub source: Entity,
    pub target: Entity,
    pub damage: usize,
}

//-------------------------------------------------------------------------------------------------------------------

/// Event emitted when an entity with a `Health` component dies.
#[derive(Debug, Deref, Clone, Copy, Event)]
pub struct EntityDeath(pub Entity);

//-------------------------------------------------------------------------------------------------------------------

#[derive(SystemSet, Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct DamageUpdateSet;

//-------------------------------------------------------------------------------------------------------------------

#[derive(SystemSet, Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub enum DamageSet
{
    DetectDamage,
    HandleDeaths,
    DespawnDead,
}

//-------------------------------------------------------------------------------------------------------------------

pub struct DamagePlugin;

impl Plugin for DamagePlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_event::<DamageEvent>()
            .add_event::<EntityDeath>()
            .configure_sets(
                Update,
                (DamageSet::DetectDamage, DamageSet::HandleDeaths, DamageSet::DespawnDead)
                    .chain()
                    .in_set(DamageUpdateSet),
            )
            .add_systems(Update, handle_damage_events.in_set(DamageSet::DetectDamage))
            .add_systems(Update, despawn_dead_entities.in_set(DamageSet::DespawnDead));
    }
}

//-------------------------------------------------------------------------------------------------------------------
