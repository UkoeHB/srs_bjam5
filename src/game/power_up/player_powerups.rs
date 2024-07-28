use bevy::prelude::*;
use bevy_cobweb::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn refresh_player_powerups(
    mut c: Commands,
    constants: ReactRes<GameConstants>,
    mut powerups: ReactResMut<PlayerPowerups>,
    bank: Res<PowerupBank>,
)
{
    tracing::info!("resetting player powerups");
    let powerups = powerups.get_mut(&mut c);
    powerups.reset();
    let Some(starting) = bank.get(&constants.starting_powerup) else {
        tracing::error!("failed setting initial powerup {:?}; powerup is unknown", constants.starting_powerup);
        return;
    };
    powerups.add(starting);
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct PowerupLevel
{
    pub name: String,
    pub level: usize,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(ReactResource, Debug, Default)]
pub struct PlayerPowerups
{
    /// Stored in a vec so they can be display in the order they were added.
    powerups: Vec<PowerupLevel>,
}

impl PlayerPowerups
{
    fn reset(&mut self)
    {
        self.powerups.clear();
    }

    pub fn add(&mut self, powerup: &PowerupInfo)
    {
        if self.powerups.iter().any(|p| p.name == powerup.name) {
            tracing::error!("ignoring attempt to add powerup that already exists {:?}", powerup);
            return;
        }
        self.powerups
            .push(PowerupLevel { name: powerup.name.clone(), level: 1 });
    }

    pub fn upgrade(&mut self, max_level: usize, name: impl AsRef<str>)
    {
        let name = name.as_ref();
        let Some(powerup) = self.powerups.iter_mut().find(|p| p.name.as_str() == name) else {
            tracing::error!("ignoring attempt to upgrade powerup that was not added {:?}", name);
            return;
        };
        if powerup.level >= max_level {
            tracing::error!("ignoring attempt to upgrade powerup {:?}, already at max level", name);
            return;
        }
        powerup.level += 1;
    }

    /// Gets the level of the requested name. Returns `0` if the player doesn't have the powerup.
    pub fn get(&self, name: impl AsRef<str>) -> usize
    {
        let name = name.as_ref();
        self.iter()
            .find(|p| p.name.as_str() == name)
            .map(|l| l.level)
            .unwrap_or_default()
    }

    pub fn iter(&self) -> impl Iterator<Item = &PowerupLevel>
    {
        self.powerups.iter()
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct PlayerPowerupPlugin;

impl Plugin for PlayerPowerupPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_react_resource::<PlayerPowerups>()
            .add_systems(OnEnter(GameState::DayStart), refresh_player_powerups);
    }
}

//-------------------------------------------------------------------------------------------------------------------
