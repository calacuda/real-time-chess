use bevy::prelude::*;

pub mod enter_room_select;
pub mod get_room_list;
pub mod handle_error;
pub mod handle_invalid_move;
pub mod handle_room_change;
pub mod recv_in_game_messages;
pub mod recv_in_room_messages;
pub mod recv_system_messages;
pub mod setup_game_camera;
pub mod update_visualizer;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Connected;
