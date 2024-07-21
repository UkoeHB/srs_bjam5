use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::sickle::ui_builder::UiBuilder;
use bevy_mod_picking::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

fn picking_barrier(
    In((entity, value)): In<(Entity, bool)>,
    mut c: Commands,
    mut entities: Query<Option<&mut Pickable>>,
)
{
    let Ok(maybe_pickable) = entities.get_mut(entity) else { return };

    match value {
        true => {
            // Case: Pickable component exists.
            if let Some(mut pickable) = maybe_pickable {
                pickable.should_block_lower = true;
                return;
            }

            // Case: Pickable component doesn't exist.
            // - Set `is_hoverable` true to match default behavior.
            c.entity(entity)
                .insert(Pickable { should_block_lower: true, is_hoverable: true });
        }
        false => {
            // Case: Pickable component exists.
            if let Some(mut pickable) = maybe_pickable {
                pickable.should_block_lower = false;
                //return;
            }

            // Case: Pickable component doesn't exist.
            // - Pick blocking is off by default.
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn pickable(In((entity, value)): In<(Entity, bool)>, mut c: Commands, mut entities: Query<Option<&mut Pickable>>)
{
    let Ok(maybe_pickable) = entities.get_mut(entity) else { return };

    match value {
        true => {
            // Case: Pickable component exists.
            if let Some(mut pickable) = maybe_pickable {
                pickable.is_hoverable = true;
                //return;
            }

            // Case: Pickable component doesn't exist.
            // - Pickability is on by default.
        }
        false => {
            // Case: Pickable component exists.
            if let Some(mut pickable) = maybe_pickable {
                pickable.is_hoverable = false;
                return;
            }

            // Case: Pickable component doesn't exist.
            c.entity(entity)
                .insert(Pickable { should_block_lower: false, is_hoverable: false });
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub trait ModPickingEntityExt
{
    /// Sets the entity to block or not block picking events from reaching entities below it.
    ///
    /// This is off by default (unless you manually added the [`Pickable`] component with
    /// `should_block_lower = true`).
    fn picking_barrier(&mut self, value: bool) -> &mut Self;
    /// Sets the entity to receive or not receive picking events.
    ///
    /// This is on by default (unless you manually added the [`Pickable`] component with `is_hoverable = false`).
    fn pickable(&mut self, value: bool) -> &mut Self;
    /// Sets the entity to not receive picking events, and to allow picking events to pass through the entity.
    fn disable_picking(&mut self) -> &mut Self;
}

impl ModPickingEntityExt for EntityCommands<'_>
{
    fn picking_barrier(&mut self, value: bool) -> &mut Self
    {
        let entity = self.id();
        self.syscall((entity, value), picking_barrier);
        self
    }

    fn pickable(&mut self, value: bool) -> &mut Self
    {
        let entity = self.id();
        self.syscall((entity, value), pickable);
        self
    }

    fn disable_picking(&mut self) -> &mut Self
    {
        self.insert(Pickable::IGNORE);
        self
    }
}

impl ModPickingEntityExt for UiBuilder<'_, Entity>
{
    fn picking_barrier(&mut self, value: bool) -> &mut Self
    {
        self.entity_commands().picking_barrier(value);
        self
    }

    fn pickable(&mut self, value: bool) -> &mut Self
    {
        self.entity_commands().pickable(value);
        self
    }

    fn disable_picking(&mut self) -> &mut Self
    {
        self.insert(Pickable::IGNORE);
        self
    }
}

//-------------------------------------------------------------------------------------------------------------------
