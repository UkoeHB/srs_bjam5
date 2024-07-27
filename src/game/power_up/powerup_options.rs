use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use rand::seq::SliceRandom;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn handle_buffered_powerups(mut c: Commands, mut buffer: ResMut<BufferedPowerUps>)
{
    if buffer.is_handling_powerup() {
        return;
    }

    if !buffer.try_start_handling() {
        return;
    }

    c.react().broadcast(PlayerPowerUp);
}

//-------------------------------------------------------------------------------------------------------------------

//todo: consider adding heuristics to improve offerings
// - avoid showing the same new options twice in a row
// - if there are lots of abilities, show upgrades in proportion to the number of filled slots
// - always have at least 1 'new' offering if there are open slots
pub fn get_powerup_options(
    constants: &GameConstants,
    rng: &mut GameRng,
    source: PowerupSource,
    player_powerups: &PlayerPowerups,
    powerup_bank: &PowerupBank,
) -> Vec<PowerupOption>
{
    // Detect how many open slots there are.
    let mut filled_passive_slots = 0;
    let mut filled_active_slots = 0;
    player_powerups
        .iter()
        .flat_map(|p| powerup_bank.get(&p.name))
        .for_each(|i| match i.ability_type {
            AbilityType::Passive => filled_passive_slots += 1,
            AbilityType::Active => filled_active_slots += 1,
        });

    let open_passive_slots = constants
        .num_passive_slots
        .saturating_sub(filled_passive_slots);
    let open_active_slots = constants
        .num_active_slots
        .saturating_sub(filled_active_slots);

    // Get candidate powerups.
    let mut candidates: Vec<PowerupOption> = powerup_bank
        .iter()
        .filter_map(|(_, i)| {
            if open_passive_slots == 0 && i.ability_type == AbilityType::Passive {
                return None;
            }
            if open_active_slots == 0 && i.ability_type == AbilityType::Active {
                return None;
            }
            let Some(powerup) = player_powerups.get(&i.name) else {
                return Some(PowerupType::New(i.name.clone()));
            };
            if powerup.level >= constants.max_powerup_level {
                return None;
            }
            Some(PowerupType::Upgrade(i.name.clone()))
        })
        .map(|p| PowerupOption::Powerup(p))
        .collect();

    // If no candidates, fall back to filler.
    if candidates.len() == 0 {
        return vec![PowerupOption::Filler(FillerType::Health), PowerupOption::Filler(FillerType::Karma)];
    }

    // Randomize.
    candidates.shuffle(rng.rng());

    // Trim excess.
    let max_offers = match source {
        PowerupSource::LevelUp => constants.max_powerup_offers,
    };
    candidates.truncate(max_offers);

    candidates
}

//-------------------------------------------------------------------------------------------------------------------

/// Types of power-up sources.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub enum PowerupSource
{
    #[default]
    LevelUp,
}

//-------------------------------------------------------------------------------------------------------------------

/// Coordinates collecting power-ups and handling them, in case multiple are collected at once.
#[derive(Resource, Default)]
pub struct BufferedPowerUps
{
    buffer: Vec<PowerupSource>,
    is_handling: bool,
}

impl BufferedPowerUps
{
    pub fn insert(&mut self, additional: impl IntoIterator<Item = PowerupSource>)
    {
        self.buffer.extend(additional);
    }

    pub fn is_handling_powerup(&self) -> bool
    {
        self.is_handling
    }

    pub fn try_start_handling(&mut self) -> bool
    {
        if self.buffer.len() == 0 {
            return false;
        }
        self.is_handling = true;
        true
    }

    pub fn current_powerup(&self) -> Option<PowerupSource>
    {
        self.buffer.get(0).cloned()
    }

    pub fn end_handling_powerup(&mut self)
    {
        if !self.is_handling {
            return;
        }
        debug_assert!(self.buffer.len() > 0);
        if self.buffer.len() > 0 {
            self.buffer.remove(0);
        }
        self.is_handling = false;
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum PowerupType
{
    New(String),
    Upgrade(String),
}

impl PowerupType
{
    pub fn apply(
        In(powerup_type): In<Self>,
        mut c: Commands,
        constants: ReactRes<GameConstants>,
        powerup_bank: Res<PowerupBank>,
        mut player_powerups: ReactResMut<PlayerPowerups>,
    )
    {
        let player_powerups = player_powerups.get_mut(&mut c);
        match powerup_type {
            Self::New(name) => {
                let Some(info) = powerup_bank.get(&name) else {
                    tracing::error!("failed adding new powerup {}, it isn't registered in PowerupBank", name);
                    return;
                };
                player_powerups.add(info);
            }
            Self::Upgrade(name) => {
                player_powerups.upgrade(constants.max_powerup_level, &name);
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum PowerupOption
{
    Powerup(PowerupType),
    Filler(FillerType),
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(SystemSet, Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct PowerUpActivateSet;

//-------------------------------------------------------------------------------------------------------------------

pub struct PowerupOptionsPlugin;

impl Plugin for PowerupOptionsPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_resource::<BufferedPowerUps>()
            .add_systems(Update, handle_buffered_powerups.in_set(PowerUpActivateSet));
    }
}

//-------------------------------------------------------------------------------------------------------------------
