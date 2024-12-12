use crate::client::{
    components::system_message::{SystemMessage, SystemMessageType},
    events::new_error::NewError,
};
use bevy::prelude::*;
use std::time::Duration;

pub fn handle_error_event(mut commands: Commands, mut error_event: EventReader<NewError>) {
    for ev in error_event.read() {
        commands.spawn(SystemMessage {
            display_duration: Duration::from_secs_f32(2.5),
            message: ev.0.clone(),
            msg_type: SystemMessageType::MiscError,
            shown: None,
        });
    }
}
