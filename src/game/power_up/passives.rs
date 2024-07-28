use std::collections::HashMap;

use bevy::ecs::world::Command;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use serde::{Deserialize, Serialize};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn refresh_player_passives(
    mut stats: Query<
        (
            &mut Health,
            &mut HealthRegen,
            &mut Armor,
            &mut CooldownReduction,
            &mut MoveSpeed,
            &mut CollectionRange,
            &mut AreaSize,
            &mut DamageAmp,
            &mut ExpAmp,
        ),
        With<Player>,
    >,
    passives: Res<PassiveDatabase>,
    player: ReactRes<PlayerPowerups>,
)
{
    let Ok((
        mut health,
        mut health_regen,
        mut armor,
        mut cdr,
        mut movespeed,
        mut collection,
        mut areasize,
        mut damageamp,
        mut expamp,
    )) = stats.get_single_mut()
    else {
        warn_once!("failed refreshing player passives, player doesn't have all passive components (WARN ONCE)");
        return;
    };

    health.set_bonus(passives.get(Passive::Health, &player));
    health_regen.set_bonus(passives.get(Passive::HealthRegen, &player));
    armor.set_bonus(passives.get(Passive::Armor, &player));
    cdr.set_bonus(passives.get(Passive::CooldownReduction, &player));
    movespeed.set_bonus(passives.get(Passive::MoveSpeed, &player));
    collection.set_bonus(passives.get(Passive::CollectionRange, &player));
    areasize.set_bonus(passives.get(Passive::AreaSize, &player));
    damageamp.set_bonus(passives.get(Passive::DamageAmp, &player));
    expamp.set_bonus(passives.get(Passive::ExpAmp, &player));
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Reflect, Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Passive
{
    /// Bonus max health per level.
    #[default]
    Health,
    /// Health regen per second, per level.
    HealthRegen,
    /// Bonus armor per level.
    Armor,
    /// Reduces cooldowns. Calculatated as `cooldown = (base_cooldown / (base_cooldown + cdr))`.
    CooldownReduction,
    /// Increases move speed. Calculated as `speed = base_speed * (1 + (move_speed / 100))`.
    MoveSpeed,
    /// Increases collection range for collectables.
    CollectionRange,
    /// Increases size of area effects. Calculated as `area*(1 + (area_size / 100))`.
    AreaSize,
    /// Amplifies damage effects. Calculated as `damage*(1 + (damage_amp / 100))`.
    DamageAmp,
    /// Amplifies how much experience is received. Calculated as `exp*(1 + (exp_amp / 100))`.
    ExpAmp,
}

impl Passive
{
    pub fn name(&self) -> &'static str
    {
        match *self {
            Self::Health => "Health",
            Self::HealthRegen => "Health Regen",
            Self::Armor => "Armor",
            Self::CooldownReduction => "Cooldown Reduction",
            Self::MoveSpeed => "Move Speed",
            Self::CollectionRange => "Collection Range",
            Self::AreaSize => "Area Size",
            Self::DamageAmp => "Damage Amp",
            Self::ExpAmp => "Exp Amp",
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Default, Reflect, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct PassiveInfo
{
    /// Bonuses per level.
    pub bonuses: Vec<usize>,
    pub icon: String,
    pub description: String,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Deref, Default, Reflect, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct PassiveDatabase(HashMap<Passive, PassiveInfo>);

impl PassiveDatabase
{
    /// Gets passive value for a given level.
    ///
    /// Returns 0 if lookup failed.
    pub fn get_for_level(&self, passive: Passive, level: usize) -> usize
    {
        self.0
            .get(&passive)
            .and_then(|info| {
                info.bonuses
                    .get(level.saturating_sub(1))
                    .or_else(|| info.bonuses.last())
            })
            .cloned()
            .unwrap_or_default()
    }

    /// Gets the current passive value to apply.
    pub fn get(&self, passive: Passive, player: &PlayerPowerups) -> usize
    {
        let level = player.get(passive.name());
        if level == 0 {
            return 0;
        }
        self.get_for_level(passive, level)
    }
}

impl Command for PassiveDatabase
{
    fn apply(self, w: &mut World)
    {
        let mut bank = w.resource_mut::<PowerupBank>();
        for (passive, info) in self.iter() {
            bank.register(PowerupInfo {
                ability_type: AbilityType::Passive,
                name: passive.name().into(),
                description: info.description.clone(),
                icon: info.icon.clone(),
            });
        }
        w.insert_resource(self);
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct PassivesPlugin;

impl Plugin for PassivesPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_command::<PassiveDatabase>()
            .init_resource::<PassiveDatabase>()
            .add_systems(Update, refresh_player_passives.in_set(PassivesUpdateSet));
    }
}

//-------------------------------------------------------------------------------------------------------------------
