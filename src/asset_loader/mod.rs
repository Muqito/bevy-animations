use crate::animated_character::{
    CharacterAnimationResource, CharacterAnimationResources, PlayerCharacterName, SceneId,
};
use bevy::gltf::Gltf;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::scene::SceneInstanceReady;
use std::time::Duration;

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, States)]
pub enum AssetLoaderState {
    #[default]
    Loading,
    Done,
}

pub struct AssetLoaderPlugin;

impl Plugin for AssetLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AssetLoaderState>();
        app.add_systems(OnEnter(AssetLoaderState::Loading), load_assets);
        app.add_systems(
            Update,
            check_for_load_complete.run_if(in_state(AssetLoaderState::Loading)),
        );
        app.add_systems(OnEnter(AssetLoaderState::Done), add_scenes);
    }
}

#[derive(Resource, Debug)]
pub struct AssetPack {
    pub gltfs: HashMap<String, Handle<Gltf>>,
}

pub fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let cleric: Handle<Gltf> = asset_server.load("Cleric.gltf");
    let dino: Handle<Gltf> = asset_server.load("triceratops/scene.gltf");

    commands.insert_resource(AssetPack {
        gltfs: HashMap::from_iter([("Cleric".to_string(), cleric), ("Dino".to_string(), dino)]),
    });
}

fn check_for_load_complete(
    asset_pack: Res<AssetPack>,
    assets_gltf: Res<Assets<Gltf>>,
    mut asset_events: MessageReader<AssetEvent<Gltf>>,
    mut next_state: ResMut<NextState<AssetLoaderState>>,
) {
    for event in asset_events.read() {
        println!("Event: {:?}", event);
        if asset_pack
            .gltfs
            .values()
            .all(|gltf| assets_gltf.contains(gltf))
        {
            println!("All loaded");
            next_state.set(AssetLoaderState::Done);
        }
    }
}

#[derive(Component)]
struct Animation {
    name: String,
    animation: AnimationNodeIndex,
    graph: Handle<AnimationGraph>,
    pub(crate) animation_player_id: Option<Entity>,
}

fn add_scenes(
    mut commands: Commands,
    asset_pack: Res<AssetPack>,
    assets_gltf: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let mut characters: HashMap<String, CharacterAnimationResource> = HashMap::new();
    for (name, gltf_handle) in &asset_pack.gltfs {
        // Get the loaded Gltf object
        if let Some(gltf) = assets_gltf.get(gltf_handle) {
            // Typically GLTFs include at least one scene
            if let Some(first_scene) = gltf.scenes.get(0) {
                let scene_handle = first_scene.clone();

                println!("Spawned scene: {}", name);

                // Get named clips (your "Idle", "Death")
                let named_clips: HashMap<String, Handle<AnimationClip>> = gltf
                    .named_animations
                    .iter()
                    .map(|(name, handle)| (name.to_string(), handle.clone()))
                    .collect();

                // Build simple graph: root → parallel leaves (one per clip)
                let mut animation_graph = AnimationGraph::new();
                let blend_node = animation_graph.add_blend(0.5, animation_graph.root);

                let mut named_nodes: HashMap<String, AnimationNodeIndex> = HashMap::new();
                for (name, clip_handle) in named_clips {
                    let node_index = animation_graph.add_clip(clip_handle, 1.0, blend_node);
                    dbg!(&name, node_index);
                    named_nodes.insert(name, node_index);
                }

                let graph = graphs.add(animation_graph);

                let scene_id = commands
                    .spawn((
                        SceneRoot(scene_handle),
                        Transform::from_xyz(0.0, 0.0, 0.0),
                        Animation {
                            name: name.to_string(),
                            animation: named_nodes.values().next().unwrap().clone(),
                            graph: graph.clone(),
                            animation_player_id: None,
                        },
                    ))
                    .id();

                commands
                    .entity(scene_id)
                    .insert(PlayerCharacterName::new(name))
                    .observe(play_animation);

                characters.insert(
                    name.to_string(),
                    CharacterAnimationResource {
                        scene_id,
                        graph,
                        named_nodes,
                        name: name.clone(),
                        animation_player_id: None,
                    },
                );
            } else {
                warn!("GLTF '{}' has no scenes.", name);
            }
        }
    }

    commands.insert_resource(CharacterAnimationResources { characters });
}

// Ref: https://github.com/mrchantey/bevy/blob/f4a37e4d49db422f4df78f5c382ae96fea09855b/examples/animation/animated_mesh.rs
fn play_animation(
    trigger: On<SceneInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    mut animations_to_play: Query<&mut Animation>,
    mut players: Query<&mut AnimationPlayer>,
    mut animation_resources: ResMut<CharacterAnimationResources>,
) {
    if let Ok(mut animation_to_play) = animations_to_play.get_mut(trigger.entity) {
        // The SceneRoot component will have spawned the scene as a hierarchy
        // of entities parented to our entity. Since the asset contained a skinned
        // mesh and animations, it will also have spawned an animation player
        // component. Search our entity's descendants to find the animation player.
        for child in children.iter_descendants(trigger.entity) {
            animation_to_play.animation_player_id = Some(child);
            if let Ok(mut player) = players.get_mut(child) {
                let mut transitions = AnimationTransitions::new();
                transitions
                    .play(&mut player, animation_to_play.animation, Duration::ZERO)
                    .repeat();

                // Help so that we more easily can find it later
                if let Some((_, character)) = animation_resources
                    .characters
                    .iter_mut()
                    .find(|(_, character)| character.scene_id == trigger.entity)
                {
                    character.animation_player_id = Some(child);
                }
                // Add the animation graph. This only needs to be done once to
                // connect the animation player to the mesh.
                commands
                    .entity(child)
                    .insert(transitions)
                    .insert(SceneId::new(trigger.entity))
                    .insert(AnimationGraphHandle(animation_to_play.graph.clone()));
            }
        }
    }
}
