use bevy::gltf::Gltf;
use bevy::prelude::*;

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
    }
}

#[derive(Resource, Debug)]
pub struct AssetPack(pub Handle<Gltf>);

pub fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handle = asset_server.load("Cleric.gltf");
    commands.insert_resource(AssetPack(handle));
}

fn check_for_load_complete(
    asset_pack: Res<AssetPack>,
    mut next_state: ResMut<NextState<AssetLoaderState>>,
    mut asset_events: MessageReader<AssetEvent<Gltf>>,
) {
    for event in asset_events.read() {
        println!("Event: {:?}", event);
        if event.is_loaded_with_dependencies(&asset_pack.0) {
            println!("Loaded Asset: {:?}", event);
            next_state.set(AssetLoaderState::Done);
        }
    }
}
