use crate::client::components::marker_components::GameCamera;
use bevy::prelude::*;

pub fn teardown_game(mut commands: Commands, game_cam: Query<Entity, With<GameCamera>>) {
    for cam in game_cam.iter() {
        commands.entity(cam).despawn();
    }
}
