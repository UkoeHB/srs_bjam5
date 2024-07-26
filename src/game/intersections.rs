use bevy::math::bounding::Aabb2d;
use bevy::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

fn update_prev_locations(mut entities: Query<(&Transform, &mut PrevLocation)>)
{
    for (transform, mut prev_location) in entities.iter_mut() {
        *prev_location = PrevLocation(transform.translation.truncate());
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Component that contains an entity's size for bounding-box intersections.
#[derive(Component, Deref, DerefMut, Debug)]
pub struct AabbSize(pub Vec2);

impl AabbSize
{
    pub fn get_2d(&self, transform: &Transform) -> Aabb2d
    {
        self.get_2d_from_vec(transform.translation.truncate())
    }

    pub fn get_2d_from_vec(&self, vec: Vec2) -> Aabb2d
    {
        Aabb2d::new(vec, self.0 / 2.)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Records an entity's location in the previous frame.
#[derive(Component, Copy, Clone, Deref, DerefMut)]
pub struct PrevLocation(pub Vec2);

//-------------------------------------------------------------------------------------------------------------------

#[derive(SystemSet, Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct PrevLocationUpdateSet;

//-------------------------------------------------------------------------------------------------------------------

pub struct IntersectionsPlugin;

impl Plugin for IntersectionsPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_systems(Update, update_prev_locations.in_set(PrevLocationUpdateSet));
    }
}

//-------------------------------------------------------------------------------------------------------------------
