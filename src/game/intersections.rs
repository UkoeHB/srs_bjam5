use bevy::prelude::*;

//use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Component that contains an entity's size for bounding-box intersections.
#[derive(Component, Deref, DerefMut)]
pub struct AabbSize(pub Vec2);

//-------------------------------------------------------------------------------------------------------------------
