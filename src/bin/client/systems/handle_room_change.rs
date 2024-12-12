use crate::client::{
    components::system_message::{SystemMessage, SystemMessageType},
    events::room_change::RoomChange,
};
use bevy::prelude::*;
use real_time_chess::display_room_id;
use std::time::Duration;

pub fn handle_room_change_event(
    mut commands: Commands,
    mut invalid_event: EventReader<RoomChange>,
    // mut sys_msg: ResMut<SystemMessages>,
) {
    for ev in invalid_event.read() {
        let (message, msg_type) = match ev {
            RoomChange::Enter(id) => (
                format!("joined room: {}", display_room_id(id)),
                SystemMessageType::RoomJoin,
            ),
            RoomChange::Exit(id) => (
                format!("left room: {}", display_room_id(id)),
                SystemMessageType::RoomLeave,
            ),
        };

        commands.spawn(SystemMessage {
            display_duration: Duration::from_secs_f32(3.5),
            message,
            msg_type,
            shown: None,
        });
    }
}
