use crate::client::states::game_state::GameState;
use bevy::prelude::*;

pub fn enter_select_room(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::RoomSelect);
}
