use bevy::math::bounding::{Aabb2d, BoundingCircle};

//-------------------------------------------------------------------------------------------------------------------

pub fn does_circle_intersect_aabb(c: &BoundingCircle, r: &Aabb2d) -> bool
{
    r.closest_point(c.center).distance_squared(c.center) <= (c.circle.radius.powi(2))
}

//-------------------------------------------------------------------------------------------------------------------
