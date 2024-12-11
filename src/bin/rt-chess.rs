use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::{EguiContexts, EguiPlugin};
use bevy_renet::netcode::{
    ClientAuthentication, NetcodeClientPlugin, NetcodeClientTransport, NetcodeTransportError,
};
use bevy_renet::{RenetClientPlugin, client_connected, renet::RenetClient};
use real_time_chess::{
    Location, PROTOCOL_ID, PlayerColor, RoomID, ServerChannel, ServerInGameMessage,
    ServerInRoomMessage, ServerSystemMessage, connection_config, display_room_id,
};
use renet_visualizer::{RenetClientVisualizer, RenetVisualizerStyle};
use std::time::{Duration, Instant};
use std::{net::UdpSocket, time::SystemTime};

#[derive(Debug, Resource)]
pub struct CurrentClientId(pub u64);

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Connected;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Event)]
pub struct InvalidMoveNotif {
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Event, Default)]
pub struct PlayerCaptureNotif;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Event, Default)]
pub struct OpponentCaptureNotif;

#[derive(Debug, Clone, Event)]
pub struct PlayerMoveNotif {
    pub from: Location,
    pub to: Location,
    pub cooldown: Duration,
}

#[derive(Debug, Clone, Event)]
pub struct OpponentMoveNotif {
    pub from: Location,
    pub to: Location,
    pub cooldown: Duration,
}

#[derive(Debug, Clone, Event)]
pub enum GameOver {
    Victory,
    Loss,
    Draw,
    OpponentDisconnect,
}

#[derive(Debug, Clone, Event)]
pub enum RoomChange {
    Enter(RoomID),
    Exit(RoomID),
}

#[derive(Debug, Clone, Event)]
pub struct ErrorNotif(pub String);

// #[derive(Debug, Clone, Event)]
// pub struct MiscError(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Component)]
pub struct RoomKey(RoomID);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SystemMessageType {
    InvalidMove,
    MiscError,
    Alert,
    RoomJoin,
    RoomLeave,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SystemMessage {
    message: String,
    msg_type: SystemMessageType,
    display_duration: Duration,
    shown: Option<Instant>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Resource)]
pub struct SystemMessages(Vec<SystemMessage>);

fn add_network(app: &mut App) {
    app.add_plugins(NetcodeClientPlugin);

    app.configure_sets(Update, Connected.run_if(client_connected));

    let client = RenetClient::new(connection_config());

    let server_addr = "127.0.0.1:5000".parse().unwrap();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let client_id = current_time.as_millis() as u64;
    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: PROTOCOL_ID,
        server_addr,
        user_data: None,
    };

    let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();

    app.insert_resource(client);
    app.insert_resource(transport);
    app.insert_resource(CurrentClientId(client_id));

    // If any error is found we just panic
    #[allow(clippy::never_loop)]
    fn panic_on_error_system(mut renet_error: EventReader<NetcodeTransportError>) {
        for e in renet_error.read() {
            error!("panicing with error: {}", e);
            panic!("{}", e);
        }
    }

    app.add_systems(Update, panic_on_error_system);
}

fn update_visulizer_system(
    mut egui_contexts: EguiContexts,
    mut visualizer: ResMut<RenetClientVisualizer<200>>,
    client: Res<RenetClient>,
    mut show_visualizer: Local<bool>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    visualizer.add_network_info(client.network_info());

    if keyboard_input.just_pressed(KeyCode::F1) {
        *show_visualizer = !*show_visualizer;
    }

    if *show_visualizer {
        visualizer.show_window(egui_contexts.ctx_mut());
    }
}

// fn client_send_player_commands(
//     mut player_commands: EventReader<PlayerCommand>,
//     mut client: ResMut<RenetClient>,
// ) {
//     for command in player_commands.read() {
//         let command_message = bincode::serialize(command).unwrap();
//         client.send_message(ClientChannel::Command, command_message);
//     }
// }

// fn recv_system_messages(
//     mut commands: Commands,
//     mut client: ResMut<RenetClient>,
//     // client_id: Res<CurrentClientId>,
//     player_color: Option<Res<PlayerColor>>,
//     mut invalid_message_event: EventWriter<InvalidMoveNotif>,
//     mut pl_capture_event: EventWriter<PlayerCaptureNotif>,
//     mut op_capture_event: EventWriter<OpponentCaptureNotif>,
//     mut player_move_event: EventWriter<PlayerMoveNotif>,
//     mut opponent_move_event: EventWriter<OpponentMoveNotif>,
//     mut game_over_event: EventWriter<GameOver>,
//     mut error_event: EventWriter<ErrorNotif>,
//     mut room_change_event: EventWriter<RoomChange>,
// ) {
//     // let client_id = client_id.0;
//     while let Some(message) = client.receive_message(ServerChannel::System) {
//         let server_message = bincode::deserialize(&message).unwrap();
//         match server_message {
//             ServerMessage::LoadingMatch(_dur) => {
//                 todo!("match loading not yet implemented");
//             }
//             // ServerMessage::ChatMessage(message) => {
//             //     info!("message recv => {message}")
//             // }
//             ServerMessage::InvalidMove(message) => {
//                 if player_color.as_ref().is_none() {
//                     continue;
//                 }
//
//                 // TODO: use system message.
//                 invalid_message_event.send(InvalidMoveNotif { message });
//             }
//             ServerMessage::ListRooms(room_ids) => {
//                 for id in room_ids {
//                     commands.spawn(RoomKey(id));
//                 }
//             }
//             ServerMessage::MoveRecv {
//                 player,
//                 from,
//                 to,
//                 capture,
//                 cooldown,
//             } => {
//                 if player_color.as_ref().is_none() {
//                     continue;
//                 }
//
//                 let my_move = player_color.as_ref().is_some_and(|color| **color == player);
//
//                 if capture && my_move {
//                     pl_capture_event.send_default();
//                 } else if capture && !my_move {
//                     op_capture_event.send_default();
//                 }
//
//                 if my_move {
//                     player_move_event.send(PlayerMoveNotif { from, to, cooldown });
//                 } else {
//                     opponent_move_event.send(OpponentMoveNotif { from, to, cooldown });
//                 }
//             }
//             ServerMessage::Victory(player) => {
//                 if player_color.as_ref().is_none() {
//                     continue;
//                 }
//
//                 let my_vic = player_color.as_ref().is_some_and(|color| **color == player);
//
//                 if my_vic {
//                     game_over_event.send(GameOver::Victory);
//                 } else {
//                     game_over_event.send(GameOver::Loss);
//                 }
//             }
//             ServerMessage::Error(message) => {
//                 error_event.send(ErrorNotif(message));
//             }
//             ServerMessage::Draw => {
//                 game_over_event.send(GameOver::Draw);
//             }
//             ServerMessage::OpponentDisconect => {
//                 game_over_event.send(GameOver::OpponentDisconnect);
//             }
//             ServerMessage::JoinedRoom(room_id) => {
//                 room_change_event.send(RoomChange::Enter(room_id));
//             }
//             ServerMessage::LeftRoom(room_id) => {
//                 room_change_event.send(RoomChange::Exit(room_id));
//             }
//         }
//     }
// }

fn recv_system_messages(
    mut commands: Commands,
    mut client: ResMut<RenetClient>,
    mut error_event: EventWriter<ErrorNotif>,
    mut room_change_event: EventWriter<RoomChange>,
) {
    // let client_id = client_id.0;
    while let Some(message) = client.receive_message(ServerChannel::System) {
        let server_message = bincode::deserialize(&message).unwrap();
        match server_message {
            ServerSystemMessage::ListRooms(room_ids) => {
                for id in room_ids {
                    commands.spawn(RoomKey(id));
                }
            }
            ServerSystemMessage::Error(message) => {
                error_event.send(ErrorNotif(message));
            }
            ServerSystemMessage::JoinedRoom(room_id) => {
                room_change_event.send(RoomChange::Enter(room_id));
            }
            ServerSystemMessage::LeftRoom(room_id) => {
                room_change_event.send(RoomChange::Exit(room_id));
            }
        }
    }
}

fn recv_in_room_messages(mut client: ResMut<RenetClient>) {
    while let Some(message) = client.receive_message(ServerChannel::InRoom) {
        let server_message = bincode::deserialize(&message).unwrap();
        match server_message {
            ServerInRoomMessage::WaitingForPlayers => {
                // TODO: switch to a "loading game" state.
                todo!("match loading not yet implemented");
            }
            ServerInRoomMessage::RoomJoinRequest(usr_name) => {
                info!("{usr_name} requested to join your room.")
                // TODO: show a pop up window that allows the room owner to accept or decline the
                // request.
            }
            ServerInRoomMessage::PlayerJoined(usr_name) => {
                info!("{usr_name} join your room.")
                // TODO: switch to the "in-game" state.
            }
        }
    }
}

fn recv_in_game_messages(
    mut client: ResMut<RenetClient>,
    player_color: Option<Res<PlayerColor>>,
    mut invalid_message_event: EventWriter<InvalidMoveNotif>,
    mut pl_capture_event: EventWriter<PlayerCaptureNotif>,
    mut op_capture_event: EventWriter<OpponentCaptureNotif>,
    mut player_move_event: EventWriter<PlayerMoveNotif>,
    mut opponent_move_event: EventWriter<OpponentMoveNotif>,
    mut game_over_event: EventWriter<GameOver>,
) {
    // let client_id = client_id.0;
    while let Some(message) = client.receive_message(ServerChannel::InGame) {
        let server_message = bincode::deserialize(&message).unwrap();
        match server_message {
            ServerInGameMessage::InvalidMove(message) => {
                if player_color.as_ref().is_none() {
                    continue;
                }

                // TODO: use system message.
                invalid_message_event.send(InvalidMoveNotif { message });
            }
            ServerInGameMessage::MoveRecv {
                player,
                from,
                to,
                capture,
                cooldown,
            } => {
                if player_color.as_ref().is_none() {
                    continue;
                }

                let my_move = player_color.as_ref().is_some_and(|color| **color == player);

                if capture && my_move {
                    pl_capture_event.send_default();
                } else if capture && !my_move {
                    op_capture_event.send_default();
                }

                if my_move {
                    player_move_event.send(PlayerMoveNotif { from, to, cooldown });
                } else {
                    opponent_move_event.send(OpponentMoveNotif { from, to, cooldown });
                }
            }
            ServerInGameMessage::Victory(player) => {
                if player_color.as_ref().is_none() {
                    continue;
                }

                let my_vic = player_color.as_ref().is_some_and(|color| **color == player);

                if my_vic {
                    game_over_event.send(GameOver::Victory);
                } else {
                    game_over_event.send(GameOver::Loss);
                }
            }
            ServerInGameMessage::Draw => {
                game_over_event.send(GameOver::Draw);
            }
            ServerInGameMessage::OpponentDisconect => {
                game_over_event.send(GameOver::OpponentDisconnect);
            }
        }
    }
}

fn handle_error_event(
    mut error_event: EventReader<ErrorNotif>,
    mut sys_msg: ResMut<SystemMessages>,
) {
    for ev in error_event.read() {
        sys_msg.0.push(SystemMessage {
            display_duration: Duration::from_secs_f32(2.5),
            message: ev.0.clone(),
            msg_type: SystemMessageType::MiscError,
            shown: None,
        });
    }
}

fn handle_invalid_move_event(
    mut invalid_event: EventReader<InvalidMoveNotif>,
    mut sys_msg: ResMut<SystemMessages>,
) {
    for ev in invalid_event.read() {
        sys_msg.0.push(SystemMessage {
            display_duration: Duration::from_secs_f32(3.5),
            message: ev.message.clone(),
            msg_type: SystemMessageType::InvalidMove,
            shown: None,
        });
    }
}

fn handle_room_change_event(
    mut invalid_event: EventReader<RoomChange>,
    mut sys_msg: ResMut<SystemMessages>,
) {
    for ev in invalid_event.read() {
        sys_msg.0.push(SystemMessage {
            display_duration: Duration::from_secs_f32(3.5),
            message: match ev {
                RoomChange::Enter(id) => format!("joined room: {}", display_room_id(id)),
                RoomChange::Exit(id) => format!("left room: {}", display_room_id(id)),
            },
            msg_type: match ev {
                RoomChange::Enter(_) => SystemMessageType::RoomJoin,
                RoomChange::Exit(_) => SystemMessageType::RoomLeave,
            },
            shown: None,
        });
    }
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(RenetClientPlugin);
    app.add_plugins(FrameTimeDiagnosticsPlugin);
    app.add_plugins(LogDiagnosticsPlugin::default());
    app.add_plugins(EguiPlugin);

    add_network(&mut app);

    // app.add_event::<PlayerCommand>();
    app.add_event::<InvalidMoveNotif>();
    app.add_event::<PlayerCaptureNotif>();
    app.add_event::<OpponentCaptureNotif>();
    app.add_event::<PlayerMoveNotif>();
    app.add_event::<OpponentMoveNotif>();
    app.add_event::<GameOver>();
    app.add_event::<ErrorNotif>();
    app.add_event::<RoomChange>();

    app.insert_resource(SystemMessages::default());

    // app.insert_resource(ClientLobby::default());
    // app.insert_resource(NetworkMapping::default());

    // app.add_systems(Update, (player_input, camera_follow, update_target_system));
    app.add_systems(
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
        )
            .in_set(Connected),
    );

    app.insert_resource(RenetClientVisualizer::<200>::new(
        RenetVisualizerStyle::default(),
    ));

    // app.add_systems(Startup, (setup_level, setup_camera, setup_target));
    app.add_systems(Update, update_visulizer_system);

    app.run();
}
