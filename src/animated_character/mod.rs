use crate::asset_loader::AssetLoaderState;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use std::ops::Deref;
use std::time::Duration;

pub struct AnimatedCharacterPlugin;

#[derive(Debug, Component)]
pub struct SceneId(Entity);
impl SceneId {
    pub fn new(id: Entity) -> Self {
        Self(id)
    }
    pub fn get_id(&self) -> Entity {
        self.0
    }
}

#[derive(Debug, Component)]
pub struct PlayerCharacterName(String);
impl PlayerCharacterName {
    pub fn new<T: Into<String>>(name: T) -> Self {
        Self(name.into())
    }
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}
// Resource: Shared graph + name-to-index mapping
#[derive(Resource)]
pub(crate) struct CharacterAnimationResource {
    pub(crate) scene_id: Entity,
    pub(crate) graph: Handle<AnimationGraph>,
    pub(crate) named_nodes: HashMap<String, AnimationNodeIndex>,
    pub(crate) name: String,
    pub(crate) animation_player_id: Option<Entity>,
}
#[derive(Resource)]
pub(crate) struct CharacterAnimationResources {
    pub(crate) characters: HashMap<String, CharacterAnimationResource>,
}
impl Plugin for AnimatedCharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            keyboard_control.run_if(in_state(AssetLoaderState::Done)),
        );
    }
}

fn keyboard_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut animation_players: Query<(&mut AnimationPlayer, &mut AnimationTransitions, &SceneId)>,
    animation_resources: Res<CharacterAnimationResources>,
    mut current_animation: Local<usize>,
    mut animate: Local<bool>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        for (mut player, _, _) in &mut animation_players {
            let Some((&playing_animation_index, _)) = player.playing_animations().next() else {
                println!("No playing animations found");
                continue;
            };
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();

            println!("playing_animation: {:?}", playing_animation);
            if *animate {
                playing_animation.pause();
            } else {
                playing_animation.resume();
            }
        }
        *animate = !*animate;
        println!("animated {}", *animate);
    }

    if keyboard_input.just_pressed(KeyCode::KeyA) {
        for (mut player, mut transitions, id) in &mut animation_players {
            let animations = animation_resources
                .characters
                .values()
                .find(|v| v.scene_id == id.get_id())
                .unwrap();
            *current_animation = (*current_animation + 1);

            let (animation_name, animation_node_index) = animations
                .named_nodes
                .iter()
                .nth(*current_animation % animations.named_nodes.len())
                .unwrap();

            println!("Change animation state to {}", animation_name);

            if player.playing_animations().next().is_none() {
                println!("No playing animations found");
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

    if keyboard_input.just_pressed(KeyCode::KeyS) {
        let Some(character) = animation_resources.characters.get("Cleric") else {
            return;
        };
        *current_animation = (*current_animation + 1) % character.named_nodes.len();

        let (animation_name, animation_node_index) = character
            .named_nodes
            .iter()
            .nth(*current_animation)
            .unwrap();

        println!("Change animation state to {}", animation_name);

        let Ok((mut player, mut transitions, _)) =
            animation_players.get_mut(character.animation_player_id.unwrap())
        else {
            println!("No playing animations found");
            return;
        };

        if player.playing_animations().next().is_none() {
            println!("No playing animations found");
            return;
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
