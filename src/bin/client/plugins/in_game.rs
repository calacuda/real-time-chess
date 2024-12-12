use crate::client::{
    states::game_state::GameState,
    systems::{
        InGame, draw_game_board::draw_game_board, draw_pieces::draw_pieces, game_setup::game_setup,
        setup_game_camera::setup_camera, teardown_game::teardown_game,
    },
};
use bevy::prelude::*;

pub struct InGamePlugin;

impl Plugin for InGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (draw_game_board, draw_pieces).in_set(InGame))
            .add_systems(OnEnter(GameState::InGame), (setup_camera, game_setup))
            .add_systems(OnExit(GameState::InGame), teardown_game)
            .configure_sets(Update, InGame.run_if(in_state(GameState::InGame)));
    }
}
