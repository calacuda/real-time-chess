use crate::client::{
    components::system_message::{SystemMessage, SystemMessageType},
    events::invalid_move::InvalidMoveNotif,
};
use bevy::prelude::*;
use std::time::Duration;

pub fn handle_invalid_move_event(
    mut commands: Commands,
    mut invalid_event: EventReader<InvalidMoveNotif>,
    // mut sys_msg: ResMut<SystemMessages>,
) {
    for ev in invalid_event.read() {
        commands.spawn(SystemMessage {
            display_duration: Duration::from_secs_f32(3.5),
            message: ev.message.clone(),
            msg_type: SystemMessageType::InvalidMove,
            shown: None,
        });
    }
}
