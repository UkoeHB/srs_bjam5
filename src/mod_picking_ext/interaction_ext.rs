use std::collections::HashMap;

use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use bevy_mod_picking::picking_core::PickSet;
use bevy_mod_picking::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Converts `bevy_mod_picking` entity events to reactive entity events (see [`ReactCommand::entity_event`]).
//todo: add more reactive methods... need a synchronized stream of input events (bevy v0.14) and need to track
// pointer state to identify pointer cancel and primary vs secondary hovers/releases
fn mod_picking_events(
    mut pressed_tracker: Local<HashMap<PointerId, Entity>>,
    mut c: Commands,
    mut presses: EventReader<Pointer<Down>>,
    mut clicks: EventReader<Pointer<Click>>,
    mut drag_ends: EventReader<Pointer<DragEnd>>,
)
{
    let mut rc = c.react();

    for event in presses.read() {
        pressed_tracker.insert(event.pointer_id, event.target);
        rc.entity_event(event.target, PickingPressed);
    }

    for event in clicks.read() {
        let Some(tracked) = pressed_tracker.get(&event.pointer_id) else { continue };
        if *tracked != event.target {
            continue;
        }
        pressed_tracker.remove(&event.pointer_id);
        rc.entity_event(event.target, PickingReleased);
    }

    // If drag ended without click, then pointer was released away from entity.
    for event in drag_ends.read() {
        let Some(tracked) = pressed_tracker.get(&event.pointer_id) else { continue };
        if *tracked != event.target {
            continue;
        }
        pressed_tracker.remove(&event.pointer_id);
        rc.entity_event(event.target, PickingCanceled);
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct PickingPressed;
pub struct PickingReleased;
pub struct PickingCanceled;

//-------------------------------------------------------------------------------------------------------------------

pub trait PickingInteractionExt
{
    fn on_pressed<M>(&mut self, callback: impl IntoSystem<(), (), M> + Send + Sync + 'static) -> &mut Self;

    fn on_released<M>(&mut self, callback: impl IntoSystem<(), (), M> + Send + Sync + 'static) -> &mut Self;

    fn on_press_canceled<M>(&mut self, callback: impl IntoSystem<(), (), M> + Send + Sync + 'static) -> &mut Self;
}

impl PickingInteractionExt for EntityCommands<'_>
{
    fn on_pressed<M>(&mut self, callback: impl IntoSystem<(), (), M> + Send + Sync + 'static) -> &mut Self
    {
        self.on_event::<PickingPressed>().r(callback);
        self
    }

    fn on_released<M>(&mut self, callback: impl IntoSystem<(), (), M> + Send + Sync + 'static) -> &mut Self
    {
        self.on_event::<PickingReleased>().r(callback);
        self
    }

    fn on_press_canceled<M>(&mut self, callback: impl IntoSystem<(), (), M> + Send + Sync + 'static) -> &mut Self
    {
        self.on_event::<PickingCanceled>().r(callback);
        self
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct PickingInteractionExtPlugin;

impl Plugin for PickingInteractionExtPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_systems(PreUpdate, mod_picking_events.after(PickSet::Focus));
    }
}

//-------------------------------------------------------------------------------------------------------------------
