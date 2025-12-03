use crate::asset_loader::{AssetLoaderState, AssetPack};
use bevy::gltf::Gltf;
use bevy::prelude::*;
use std::collections::HashMap;
use std::time::Duration;

pub struct AnimatedCharacterPlugin;

#[derive(Debug, Component)]
pub struct PlayerCharacterName(String);
impl PlayerCharacterName {
    pub fn new<T: Into<String>>(name: T) -> Self {
        Self(name.into())
    }
}
// Resource: Shared graph + name-to-index mapping
#[derive(Resource)]
struct CharacterAnimationResources {
    graph: Handle<AnimationGraph>,
    named_nodes: HashMap<String, AnimationNodeIndex>,
}
impl Plugin for AnimatedCharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AssetLoaderState::Done), spawn_characters);
        app.add_systems(
            Update,
            run_animations.run_if(in_state(AssetLoaderState::Done)),
        );
        app.add_systems(
            Update,
            keyboard_control.run_if(in_state(AssetLoaderState::Done)),
        );
    }
}

fn spawn_characters(
    mut commands: Commands,
    asset_pack: Res<AssetPack>,
    assets_gltf: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let gltf = assets_gltf.get(&asset_pack.0).unwrap();

    commands.spawn((
        SceneRoot(gltf.named_scenes["Scene"].clone()),
        Transform::from_xyz(0.0, 0.0, 0.0),
        PlayerCharacterName::new("one"),
    ));
    commands.spawn((
        SceneRoot(gltf.named_scenes["Scene"].clone()),
        Transform::from_xyz(2.0, 0.0, 0.0),
        PlayerCharacterName::new("two"),
    ));

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

    let handle = graphs.add(animation_graph);

    commands.insert_resource(CharacterAnimationResources {
        graph: handle,
        named_nodes,
    });
}

fn run_animations(
    mut commands: Commands,
    mut animation_player_query: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
    animations: Res<CharacterAnimationResources>,
) {
    for (entity, mut player) in &mut animation_player_query {
        //let player_name = PlayerCharacterName::new("one");
        let mut transitions = AnimationTransitions::new();
        transitions
            .play(
                &mut player,
                animations.named_nodes.get("Idle").unwrap().to_owned(),
                Duration::ZERO,
            )
            .repeat()
            .set_speed(10.0);

        commands
            .entity(entity)
            .insert(AnimationGraphHandle(animations.graph.clone()))
            .insert(transitions);
    }
}

fn keyboard_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut animation_players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    animations: Res<CharacterAnimationResources>,
    mut current_animation: Local<usize>,
) {
    for (mut player, _) in &mut animation_players {
        let Some((&playing_animation_index, _)) = player.playing_animations().next() else {
            continue;
        };
        if keyboard_input.just_pressed(KeyCode::Space) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            if playing_animation.is_paused() {
                playing_animation.resume();
            } else {
                playing_animation.pause();
            }
        }
    }
    if keyboard_input.just_pressed(KeyCode::KeyS) {
        *current_animation = (*current_animation + 1) % animations.named_nodes.len();

        let (animation_name, animation_node_index) = animations
            .named_nodes
            .iter()
            .nth(*current_animation)
            .unwrap();

        println!("Change animation state to {}", animation_name);

        for (mut player, mut transitions) in &mut animation_players {
            if player.playing_animations().next().is_none() {
                continue;
            };

            transitions
                .play(
                    &mut player,
                    *animation_node_index,
                    Duration::from_millis(250),
                )
                .repeat();
        }
    }
}
