use bevy::prelude::*;
use bevy_cobweb::prelude::*;

//use crate::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(ReactResource, Debug)]
pub struct Day
{
    current: usize,
}

impl Day
{
    pub fn set(&mut self, day: usize)
    {
        self.current = day;
    }

    pub fn increment(&mut self)
    {
        self.current += 1;
    }

    pub fn get(&self) -> usize
    {
        self.current
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct DayPlugin;

impl Plugin for DayPlugin
{
    fn build(&self, app: &mut App)
    {
        app.insert_react_resource(Day { current: 1 });
    }
}

//-------------------------------------------------------------------------------------------------------------------
