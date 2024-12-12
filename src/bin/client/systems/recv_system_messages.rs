use crate::client::{
    components::room_key::RoomKey,
    events::{new_error::NewError, room_change::RoomChange},
};
use bevy::prelude::*;
use bevy_renet::renet::RenetClient;
use real_time_chess::{ServerChannel, ServerSystemMessage};

pub fn recv_system_messages(
    mut commands: Commands,
    mut client: ResMut<RenetClient>,
    mut error_event: EventWriter<NewError>,
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
                error_event.send(NewError(message));
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
