use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::EguiPlugin;
use bevy_renet::RenetClientPlugin;
use client::{
    events::{
        game_end::GameEnd, invalid_move::InvalidMoveNotif, new_error::NewError,
        opponent_capture::OpponentCaptureNotif, opponent_move::OpponentMoveNotif,
        player_capture::PlayerCaptureNotif, player_move::PlayerMoveNotif, room_change::RoomChange,
    },
    plugins::setup_network_plugin::SetupNetwork,
    states::game_state::GameState,
    systems::{
        Connected, enter_room_select::enter_select_room, get_room_list::get_rooms_list,
        handle_error::handle_error_event, handle_invalid_move::handle_invalid_move_event,
        handle_room_change::handle_room_change_event, recv_in_game_messages::recv_in_game_messages,
        recv_in_room_messages::recv_in_room_messages, recv_system_messages::recv_system_messages,
        setup_game_camera::setup_camera, update_visualizer::update_visulizer_system,
    },
};
use renet_visualizer::{RenetClientVisualizer, RenetVisualizerStyle};

pub mod client;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RenetClientPlugin)
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(EguiPlugin)
        .add_plugins(SetupNetwork)
        .init_state::<GameState>()
        // app.add_event::<PlayerCommand>()
        .add_event::<InvalidMoveNotif>()
        .add_event::<PlayerCaptureNotif>()
        .add_event::<OpponentCaptureNotif>()
        .add_event::<PlayerMoveNotif>()
        .add_event::<OpponentMoveNotif>()
        .add_event::<GameEnd>()
        .add_event::<NewError>()
        .add_event::<RoomChange>()
        // app.add_systems(Update, (player_input, camera_follow, update_target_system));
        .add_systems(
            Update,
            (
                // client_send_input,
                // client_send_player_commands,
                recv_system_messages,
                recv_in_room_messages,
                recv_in_game_messages,
                handle_error_event,
                handle_invalid_move_event,
                handle_room_change_event,
                update_visulizer_system,
            )
                .in_set(Connected),
        )
        .add_systems(OnEnter(GameState::RoomSelect), get_rooms_list)
        .add_systems(
            Update,
            enter_select_room
                .run_if(in_state(GameState::Startup))
                .in_set(Connected),
        )
        .insert_resource(RenetClientVisualizer::<200>::new(
            RenetVisualizerStyle::default(),
        ))
        .add_systems(Startup, setup_camera)
        .run();
}
