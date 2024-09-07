use std::collections::HashMap;
use std::marker::PhantomData;

use bevy::prelude::*;
use bevy::state::state::FreelyMutableState;

//-------------------------------------------------------------------------------------------------------------------

fn clear_event_queue<E: Event>(w: &mut World)
{
    w.get_resource_mut::<Events<E>>()
        .map(|mut queue| queue.clear());
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource)]
struct StateScopedEvents<S: FreelyMutableState>
{
    cleanup_fns: HashMap<S, Vec<fn(&mut World)>>,
}

impl<S: FreelyMutableState> StateScopedEvents<S>
{
    fn add_event<E: Event>(&mut self, state: S)
    {
        self.cleanup_fns
            .entry(state)
            .or_default()
            .push(clear_event_queue::<E>);
    }

    fn cleanup(&self, w: &mut World, state: S)
    {
        let Some(fns) = self.cleanup_fns.get(&state) else { return };
        for callback in fns {
            (*callback)(w);
        }
    }
}

impl<S: FreelyMutableState> Default for StateScopedEvents<S>
{
    fn default() -> Self
    {
        Self { cleanup_fns: HashMap::default() }
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn cleanup_state_scoped_event<S: FreelyMutableState>(
    mut c: Commands,
    mut transitions: EventReader<StateTransitionEvent<S>>,
)
{
    let Some(transition) = transitions.read().last() else { return };
    if transition.entered == transition.exited {
        return;
    }
    let Some(exited) = transition.exited.clone() else { return };

    c.add(move |w: &mut World| {
        w.resource_scope::<StateScopedEvents<S>, ()>(|w, events| {
            events.cleanup(w, exited);
        });
    });
}

//-------------------------------------------------------------------------------------------------------------------

fn add_state_scoped_event_impl<E: Event, S: FreelyMutableState>(app: &mut App, _p: PhantomData<E>, state: S)
{
    if !app.world().contains_resource::<StateScopedEvents<S>>() {
        app.init_resource::<StateScopedEvents<S>>();
    }
    app.add_event::<E>();
    app.world_mut()
        .resource_mut::<StateScopedEvents<S>>()
        .add_event::<E>(state.clone());
    app.add_systems(OnExit(state), cleanup_state_scoped_event::<S>);
}

//-------------------------------------------------------------------------------------------------------------------

pub trait StateScopedEventsExt
{
    /// Adds a `bevy` event type that is automatically cleaned up when leaving state `S`.
    ///
    /// Note that event cleanup is ordered ambiguously relative to `StateScoped` entity cleanup and the `OnExit`
    /// schedule for the target state. All of these (state scoped entities and events cleanup, and `OnExit`) occur
    /// within schedule `StateTransition` and system set `StateTransitionSteps::ExitSchedules`.
    fn add_state_scoped_event<E: Event>(&mut self, state: impl FreelyMutableState) -> &mut Self;
}

impl StateScopedEventsExt for App
{
    fn add_state_scoped_event<E: Event>(&mut self, state: impl FreelyMutableState) -> &mut Self
    {
        add_state_scoped_event_impl(self, PhantomData::<E>::default(), state);
        self
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub trait CommandsStatesExt
{
    /// Convenience method for setting a `bevy` state.
    fn set_state<S: FreelyMutableState>(&mut self, state: S);
}

impl CommandsStatesExt for Commands<'_, '_>
{
    fn set_state<S: FreelyMutableState>(&mut self, state: S)
    {
        self.add(move |w: &mut World| {
            let mut next = w.resource_mut::<NextState<S>>();
            if let NextState::Pending(prev) = &*next {
                tracing::debug!("overwriting state {:?} with {:?}", prev, state);
            }
            next.set(state);
        });
    }
}

//-------------------------------------------------------------------------------------------------------------------
