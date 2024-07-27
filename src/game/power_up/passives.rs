use bevy::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Reflect, Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Passive
{
    // Bonus max health per level.
    #[default]
    Health,
    // Health regen per second, per level.
    HealthRegen,
    // Bonus armor per level.
    Armor,
    // Reduces cooldowns, calculatated as `cooldown = (base_cooldown / (base_cooldown + cdr))`
    CooldownReduction,
    /// Increases move speed, calculated as `speed = base_speed * (1 + (move_speed / 100))
    MoveSpeed,
    /// Increases collection range for collectables.
    CollectionRange,
    /// Increases size of area effects. Calculated as area*(1 + (area_size / 100))
    AreaSize,
    /// Amplifies damage effects. Calculated as damage*(1 + (damage_amp / 100))
    DamageAmp
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Default, Reflect, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct PassiveInfo
{
    /// Bonuses per level.
    pub bonuses: Vec<usize>,
    pub name: String,
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
        .and_then(|info| info.bonuses.get(level.saturating_sub(1)).or_else(|| info.bonuses.last()))
        .unwrap_or_default()
    }
}

impl Command for PassiveDatabase
{
    fn apply(self, w: &mut World)
    {
        let bank = w.resource_mut::<PowerupBank>();
        for (_passive, info) in self.iter() {
            bank.register(PowerupInfo {
                ability_type: AbilityType::Passive,
                name: info.name.clone(),
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
            .init_resource::<PassiveDatabase>();
    }
}

//-------------------------------------------------------------------------------------------------------------------
