use bevy::prelude::*;
use bevy_renet::renet::RenetClient;
use real_time_chess::{ClientChannel, ClientSystemMessage};

pub fn get_rooms_list(mut client: ResMut<RenetClient>) {
    client.send_message(
        ClientChannel::System,
        bincode::serialize(&ClientSystemMessage::ListRooms).unwrap(),
    )
}
