use bevy::prelude::*;
use real_time_chess::RoomID;

#[derive(Debug, Clone, Event)]
pub enum RoomChange {
    Enter(RoomID),
    Exit(RoomID),
}
