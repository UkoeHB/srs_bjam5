use bevy::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn handle_damage_events(
    mut c: Commands,
    mut events: EventReader<DamageEvent>,
    mut targets: Query<(&mut Health, &Armor)>,
)
{
    for DamageEvent { target, damage } in events.read() {
        let Ok((mut hp, armor)) = targets.get_mut(*target) else { continue };

        // Check if entity is already dead.
        if hp.current == 0 {
            continue;
        }

        // Calculate damage to apply.
        let damage = ((*damage as f32) * (100. / (armor.armor as f32 + 100.))) as usize;
        hp.remove(damage);

        // Check for entity death.
        if hp.current == 0 {
            c.trigger_targets(EntityDeath, *target);
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Event sent to apply damage to an entity.
#[derive(Event, Debug, Copy, Clone)]
pub struct DamageEvent
{
    pub target: Entity,
    pub damage: usize,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(SystemSet, Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct DamageUpdateSet;

//-------------------------------------------------------------------------------------------------------------------

pub struct DamagePlugin;

impl Plugin for DamagePlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_event::<DamageEvent>()
            .add_systems(Update, handle_damage_events.in_set(DamageUpdateSet));
    }
}

//-------------------------------------------------------------------------------------------------------------------