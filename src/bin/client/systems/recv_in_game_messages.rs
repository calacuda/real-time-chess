use crate::client::events::{
    game_end::GameEnd, invalid_move::InvalidMoveNotif, opponent_capture::OpponentCaptureNotif,
    opponent_move::OpponentMoveNotif, player_capture::PlayerCaptureNotif,
    player_move::PlayerMoveNotif,
};
use bevy::prelude::*;
use bevy_renet::renet::RenetClient;
use real_time_chess::{PlayerColor, ServerChannel, ServerInGameMessage};

pub fn recv_in_game_messages(
    mut client: ResMut<RenetClient>,
    player_color: Option<Res<PlayerColor>>,
    mut invalid_message_event: EventWriter<InvalidMoveNotif>,
    mut pl_capture_event: EventWriter<PlayerCaptureNotif>,
    mut op_capture_event: EventWriter<OpponentCaptureNotif>,
    mut player_move_event: EventWriter<PlayerMoveNotif>,
    mut opponent_move_event: EventWriter<OpponentMoveNotif>,
    mut game_over_event: EventWriter<GameEnd>,
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
                    // TODO: send player capture notification.
                    // pl_capture_event.send(PlayerCaptureNotif { piece: , square: to });
                } else if capture && !my_move {
                    // TODO: send opponent capture notification.
                    // op_capture_event.send_default();
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
                    game_over_event.send(GameEnd::Victory);
                } else {
                    game_over_event.send(GameEnd::Loss);
                }
            }
            ServerInGameMessage::Draw => {
                game_over_event.send(GameEnd::Draw);
            }
            ServerInGameMessage::OpponentDisconect => {
                game_over_event.send(GameEnd::OpponentDisconnect);
            }
        }
    }
}
