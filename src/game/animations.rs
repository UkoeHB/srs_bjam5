use bevy::prelude::*;
use bevy_spritesheet_animation::events::AnimationEvent;

//todo: poll for animation marker events and handle the ones that are known ??

//-------------------------------------------------------------------------------------------------------------------

fn handle_animation_death_events(
    mut c: Commands,
    mut events: EventReader<AnimationEvent>,
    killme: Query<&DespawnOnAnimationCycle>,
)
{
    for event in events.read() {
        let AnimationEvent::AnimationCycleEnd { entity, .. } = event else { continue };
        if !killme.contains(*entity) {
            continue;
        }

        c.entity(*entity).despawn_recursive();
    }
}

//-------------------------------------------------------------------------------------------------------------------

//AnimationEvent::AnimationCycleEnd

#[derive(Component, Debug)]
pub struct DespawnOnAnimationCycle;

//-------------------------------------------------------------------------------------------------------------------

pub struct AnimationEventsPlugin;

impl Plugin for AnimationEventsPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_systems(PreUpdate, handle_animation_death_events);
    }
}

//-------------------------------------------------------------------------------------------------------------------
