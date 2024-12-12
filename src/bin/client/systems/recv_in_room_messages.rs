use bevy::prelude::*;
use bevy_renet::renet::RenetClient;
use real_time_chess::{ServerChannel, ServerInRoomMessage};

pub fn recv_in_room_messages(mut client: ResMut<RenetClient>) {
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
