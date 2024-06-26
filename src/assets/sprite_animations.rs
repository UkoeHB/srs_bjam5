use std::collections::HashMap;
use std::sync::Arc;

use bevy::ecs::system::EntityCommands;
use bevy::ecs::world::Command;
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use bevy_spritesheet_animation::prelude::*;
use serde::{Deserialize, Serialize};

//-------------------------------------------------------------------------------------------------------------------

fn setup_animation(
    image_map: &ImageMap,
    library: &mut SpritesheetLibrary,
    atlas_layouts: &mut Assets<TextureAtlasLayout>,
    animations: &mut SpriteAnimations,
    spritesheet_markers: &mut SpritesheetMarkers,
    mut animation: SpriteAnimation,
)
{
    // Animation clips.
    let clips: Vec<(SpriteAnimationClip, Vec<(AnimationMarkerId, usize)>)> = animation
        .clips
        .drain(..)
        .map(|mut clip| {
            let markers: Vec<(AnimationMarkerId, usize)> = clip
                .markers
                .drain(..)
                .map(|(name, frame)| {
                    let marker_id = library.new_marker();
                    spritesheet_markers.insert(marker_id, name);
                    (marker_id, frame)
                })
                .collect();

            (clip, markers)
        })
        .collect();

    let clip_ids: Vec<AnimationClipId> = clips
        .iter()
        .map(|(clip, markers)| {
            library.new_clip(|builder| {
                match clip.frames.clone() {
                    AnimationFrames::Row(row) => {
                        builder.push_frame_indices(
                            Spritesheet::new(animation.columns as usize, animation.rows as usize).row(row),
                        );
                    }
                    AnimationFrames::Column(column) => {
                        builder.push_frame_indices(
                            Spritesheet::new(animation.columns as usize, animation.rows as usize).column(column),
                        );
                    }
                    AnimationFrames::Frame(frame) => {
                        builder.push_frame_indices(vec![frame]);
                    }
                    AnimationFrames::Frames(frames) => {
                        builder.push_frame_indices(frames);
                    }
                }

                for (marker_id, marker_frame) in markers.iter() {
                    builder.add_marker(*marker_id, *marker_frame);
                }
            })
        })
        .collect();

    // Animation.
    let anim_id = library.new_animation(|builder| {
        // Set frame duration.
        builder.set_duration(AnimationDuration::PerFrame(animation.frame_time));

        // Set cycles.
        if let Some(cycles) = animation.loops {
            builder.set_repeat(AnimationRepeat::Cycles(cycles as u32));
        }

        // Add clips.
        for clip_id in clip_ids.iter() {
            builder.add_stage((*clip_id).into());
        }
    });

    // Save the animation's sprite info.
    let image = image_map.get(animation.image);
    let layout = atlas_layouts.add(TextureAtlasLayout::from_grid(
        animation.size,
        animation.columns,
        animation.rows,
        animation.padding,
        animation.offset,
    ));
    animations.insert(anim_id, animation.name, image, layout);
}

//-------------------------------------------------------------------------------------------------------------------

fn load_sprite_animations(
    In(loaded): In<Vec<SpriteAnimation>>,
    image_map: Res<ImageMap>,
    mut library: ResMut<SpritesheetLibrary>,
    mut spritesheet_markers: ResMut<SpritesheetMarkers>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut animations: ResMut<SpriteAnimations>,
)
{
    for animation in loaded {
        setup_animation(
            &image_map,
            &mut library,
            &mut atlas_layouts,
            &mut animations,
            &mut spritesheet_markers,
            animation,
        );
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Forward marker events from the *previous frame* as entity events.
///
/// We add a 1-frame delay so the marker frame can actually be displayed before we react to it.
//todo: is this 1-frame delay correct?
fn forward_marker_events(
    mut c: Commands,
    mut events: EventReader<AnimationEvent>,
    markers: Res<SpritesheetMarkers>,
)
{
    for event in events.read() {
        let AnimationEvent::MarkerHit { entity, marker_id, animation_id, stage_index: _ } = event else {
            continue;
        };
        let name = markers
            .get(*marker_id)
            .expect("all marker ids should be registered");
        c.react()
            .entity_event(*entity, AnimationMarker::new(name.clone(), *animation_id))
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn set_sprite_animation(
    In((entity, animation)): In<(Entity, SpritesheetAnimation)>,
    mut c: Commands,
    sprite_animations: Res<SpriteAnimations>,
    sprites: Query<(), With<Sprite>>,
    transforms: Query<(), With<Transform>>,
)
{
    let Some(mut ec) = c.get_entity(entity) else { return };
    let (_, image, layout) = sprite_animations.get(animation.animation_id);
    ec.try_insert(animation);
    ec.try_insert(image);
    ec.try_insert(TextureAtlas { layout, ..default() }); // Index controlled by bevy_spritesheet_animation
    if !sprites.contains(entity) {
        ec.try_insert(Sprite::default());
    }
    if !transforms.contains(entity) {
        tracing::warn!("sprite animation set on entity {entity:?} with no Transform; add SpatialBundle to the \
            entity");
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Reflect, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnimationFrames
{
    Row(usize),
    Column(usize),
    Frame(usize),
    Frames(Vec<usize>),
}

impl Default for AnimationFrames
{
    fn default() -> Self
    {
        AnimationFrames::Row(0)
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpriteAnimationClip
{
    pub frames: AnimationFrames,
    /// Frame markers which will cause a [`MarkerHit`] event to be emitted.
    ///
    /// Contains a vector of marker names and frame indices.
    #[reflect(default)]
    pub markers: Vec<(String, usize)>,
}

//-------------------------------------------------------------------------------------------------------------------

//todo: can add more info like animation settings and more precise ways to extract spritesheets
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpriteAnimation
{
    pub name: String,
    pub image: String,
    pub size: UVec2,
    pub columns: u32,
    pub rows: u32,
    #[reflect(default)]
    pub padding: Option<UVec2>,
    #[reflect(default)]
    pub offset: Option<UVec2>,
    /// Frame time in milliseconds.
    pub frame_time: u32,
    pub clips: Vec<SpriteAnimationClip>,
    /// The number of times the animation loops when it plays.
    ///
    /// Defaults to `None`, which means infinite.
    #[reflect(default)]
    pub loops: Option<usize>,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Default)]
pub struct SpriteAnimations
{
    names: HashMap<String, AnimationId>,
    map: HashMap<AnimationId, (String, Handle<Image>, Handle<TextureAtlasLayout>)>,
}

impl SpriteAnimations
{
    /// Adds an spritesheet that should be loaded.
    ///
    /// Note that the `SpriteAnimations` does *not* track whether the `image` handle is loaded or not. It is
    /// assumed the handle was obtained from [`ImageMap`], which impliments [`LoadProgress`].
    pub fn insert(
        &mut self,
        animation_id: AnimationId,
        image_name: impl AsRef<str> + Into<String>,
        image: Handle<Image>,
        texture_layout: Handle<TextureAtlasLayout>,
    )
    {
        if self.names.contains_key(image_name.as_ref()) {
            tracing::warn!("overwriting spritesheet map entry {}", image_name.as_ref());
        }

        let image_name = image_name.into();
        self.names.insert(image_name.clone(), animation_id);
        self.map
            .insert(animation_id, (image_name, image, texture_layout));
    }

    /// Gets spritesheet handles for the given image id.
    ///
    /// Panics if the animation was not pre-inserted via [`Self::insert`].
    pub fn get(&self, animation_id: AnimationId) -> (&String, Handle<Image>, Handle<TextureAtlasLayout>)
    {
        let Some(entry) = self.map.get(&animation_id) else {
            panic!("failed getting spritesheet with unknown animation id {:?}; use LoadSpriteAnimations command",
                animation_id);
        };
        (&entry.0, entry.1.clone(), entry.2.clone())
    }

    /// Gets the animation ID for a given animation name.
    ///
    /// Returns `None` if unknown.
    pub fn get_id(&self, name: impl AsRef<str>) -> Option<AnimationId>
    {
        self.names.get(name.as_ref()).cloned()
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Entity event sent when a marker for an animation on the entity is reached.
pub struct AnimationMarker
{
    pub name: Arc<str>,
    pub animation_id: AnimationId,
}

impl AnimationMarker
{
    pub fn new(name: Arc<str>, animation_id: AnimationId) -> Self
    {
        Self { name, animation_id }
    }

    pub fn equals(&self, other: impl AsRef<str>) -> bool
    {
        &*self.name == other.as_ref()
    }
}

#[derive(Resource, Default)]
pub struct SpritesheetMarkers
{
    markers: HashMap<AnimationMarkerId, Arc<str>>,
}

impl SpritesheetMarkers
{
    pub(crate) fn insert(&mut self, id: AnimationMarkerId, name: impl AsRef<str>)
    {
        self.markers.insert(id, name.as_ref().into());
    }

    pub fn get(&self, id: AnimationMarkerId) -> Option<&Arc<str>>
    {
        self.markers.get(&id)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Loadable command for registering spritesheet assets that need to be pre-loaded.
///
/// The loaded spritesheets can be accessed via [`SpriteAnimations`].
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoadSpriteAnimations(pub Vec<SpriteAnimation>);

impl Command for LoadSpriteAnimations
{
    fn apply(self, w: &mut World)
    {
        // Load images into ImageMap first.
        let images: Vec<LoadedImage> = self
            .0
            .iter()
            .map(|config| LoadedImage { image: config.image.clone(), ..default() })
            .collect();
        LoadImages(images).apply(w);

        // Now load the animations.
        w.syscall(self.0, load_sprite_animations);
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub trait SpriteAnimationEntityCommandsExt
{
    /// Sets a spritesheet animation on the entity with default animation settings (start playing immediately
    /// with default speed).
    ///
    /// The entity must have the components in [`SpatialBundle`] for this to work.
    fn set_sprite_animation(
        &mut self,
        animations: &SpriteAnimations,
        animation_name: impl AsRef<str>,
    ) -> &mut Self;

    /// Sets a spritesheet animation with custom animation settings.
    ///
    /// The entity must have the components in [`SpatialBundle`] for this to work.
    fn set_sprite_animation_with(&mut self, animation: SpritesheetAnimation) -> &mut Self;
}

impl SpriteAnimationEntityCommandsExt for EntityCommands<'_>
{
    fn set_sprite_animation(&mut self, animations: &SpriteAnimations, animation_name: impl AsRef<str>)
        -> &mut Self
    {
        let entity = self.id();
        let Some(animation_id) = animations.get_id(animation_name.as_ref()) else {
            tracing::warn!("failed setting spritesheet animation {} on {entity:?}, animation is not registered in \
                SpriteAnimations", animation_name.as_ref());
            return self;
        };

        let mut animation = SpritesheetAnimation::from_id(animation_id);
        animation.playing = true;
        animation.speed_factor = 1.0;
        animation.reset();
        self.set_sprite_animation_with(animation)
    }

    fn set_sprite_animation_with(&mut self, animation: SpritesheetAnimation) -> &mut Self
    {
        let entity = self.id();
        self.commands()
            .syscall((entity, animation), set_sprite_animation);
        self
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct SpriteAnimationLoadPlugin;

impl Plugin for SpriteAnimationLoadPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_resource::<SpriteAnimations>()
            .init_resource::<SpritesheetMarkers>()
            .register_type::<SpriteAnimation>()
            .register_type::<Vec<SpriteAnimation>>()
            .register_type::<Option<usize>>()
            .register_type::<Option<Vec2>>()
            .register_type::<Vec<(String, usize)>>()
            .register_type::<(String, usize)>()
            .register_type::<AnimationFrames>()
            .register_type::<SpriteAnimationClip>()
            .register_type::<Vec<SpriteAnimationClip>>()
            .register_command::<LoadSpriteAnimations>()
            // PreUpdate means we react to events from the previous frame, which are emitted in PostUpdate.
            .add_systems(PreUpdate, forward_marker_events);
    }
}

//-------------------------------------------------------------------------------------------------------------------
