use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// This will make extra assets if `GameConstants` is mutated multiple times, but it should only mutate once
/// in prod.
fn insert_billboard_cache(
    mut c: Commands,
    constants: ReactRes<GameConstants>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
)
{
    c.insert_resource(BillboardCache {
        hp_bar_mesh: meshes.add(Rectangle::from_size(constants.hp_bar_size)),
        exp_bar_mesh: meshes.add(Rectangle::from_size(constants.exp_bar_size)),
        hp_bar_filled_color: materials.add(constants.hp_bar_filled_color),
        hp_bar_empty_color: materials.add(constants.hp_bar_empty_color),
        exp_bar_filled_color: materials.add(constants.exp_bar_filled_color),
        exp_bar_empty_color: materials.add(constants.exp_bar_empty_color),
    });
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource)]
pub struct BillboardCache
{
    hp_bar_mesh: Handle<Mesh>,
    exp_bar_mesh: Handle<Mesh>,
    hp_bar_filled_color: Handle<ColorMaterial>,
    hp_bar_empty_color: Handle<ColorMaterial>,
    exp_bar_filled_color: Handle<ColorMaterial>,
    exp_bar_empty_color: Handle<ColorMaterial>,
}

impl BillboardCache
{
    pub fn hp_bar_mesh(&self) -> Handle<Mesh>
    {
        self.hp_bar_mesh.clone()
    }

    pub fn exp_bar_mesh(&self) -> Handle<Mesh>
    {
        self.exp_bar_mesh.clone()
    }

    pub fn hp_bar_filled_color(&self) -> Handle<ColorMaterial>
    {
        self.hp_bar_filled_color.clone()
    }

    pub fn hp_bar_empty_color(&self) -> Handle<ColorMaterial>
    {
        self.hp_bar_empty_color.clone()
    }

    pub fn exp_bar_filled_color(&self) -> Handle<ColorMaterial>
    {
        self.exp_bar_filled_color.clone()
    }

    pub fn exp_bar_empty_color(&self) -> Handle<ColorMaterial>
    {
        self.exp_bar_empty_color.clone()
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct BillboardCachePlugin;

impl Plugin for BillboardCachePlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_systems(OnExit(LoadState::Loading), insert_billboard_cache)
            .react(|rc| rc.on_persistent(resource_mutation::<GameConstants>(), insert_billboard_cache));
    }
}

//-------------------------------------------------------------------------------------------------------------------
