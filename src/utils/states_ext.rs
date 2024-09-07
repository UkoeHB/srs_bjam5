use bevy::prelude::*;
use bevy::state::state::FreelyMutableState;

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
            w.resource_mut::<NextState<S>>().set(state);
        });
    }
}

//-------------------------------------------------------------------------------------------------------------------
