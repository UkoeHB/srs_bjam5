use bevy::prelude::*;
use bevy_cobweb::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn spawn_player(mut c: Commands, constants: ReactRes<GameConstants>, animations: Res<SpriteAnimations>)
{
    c.spawn((
        Player { health: constants.player_base_hp },
        SpatialBundle::from_transform(Transform::default()),
        SpriteLayer::Objects,
        //PlayerDirection::Up,
        //Action::Standing,
        //AabbSize(constants.player_size),
        //todo: scoping to GameState::Play means the player despawns on entering GameState::DayOver, even though
        // we may want to continue displaying the player in the background
        StateScoped(GameState::Play),
    ))
    .set_sprite_animation(&animations, &constants.player_standing_animation);
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Debug)]
pub struct Player
{
    pub health: usize,
}

//-------------------------------------------------------------------------------------------------------------------

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_systems(OnEnter(GameState::Play), spawn_player);
    }
}

//-------------------------------------------------------------------------------------------------------------------
