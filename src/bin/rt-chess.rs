use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    log::LogPlugin,
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
    plugins::{in_game::InGamePlugin, setup_network_plugin::SetupNetwork},
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
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "rt-chess".into(),
                        // resolution: (640.0, 480.0).into(),
                        resizable: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    watch_for_changes_override: Some(true),
                    ..Default::default()
                })
                .set(LogPlugin {
                    // level: bevy::log::Level::DEBUG,
                    // filter: "debug,wgpu_core=warn,wgpu_hal=warn".into(),
                    ..default()
                })
                .build(),
        )
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(RenetClientPlugin)
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(EguiPlugin)
        .add_plugins(SetupNetwork)
        .add_plugins(InGamePlugin)
        .init_state::<GameState>()
        .add_event::<InvalidMoveNotif>()
        .add_event::<PlayerCaptureNotif>()
        .add_event::<OpponentCaptureNotif>()
        .add_event::<PlayerMoveNotif>()
        .add_event::<OpponentMoveNotif>()
        .add_event::<GameEnd>()
        .add_event::<NewError>()
        .add_event::<RoomChange>()
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
                enter_select_room.run_if(in_state(GameState::Startup)),
            )
                .in_set(Connected),
        )
        .add_systems(OnEnter(GameState::RoomSelect), get_rooms_list)
        .insert_resource(RenetClientVisualizer::<200>::new(
            RenetVisualizerStyle::default(),
        ))
        .run();
}
