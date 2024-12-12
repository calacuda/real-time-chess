use bevy::prelude::*;
use real_time_chess::RoomID;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Component)]
pub struct RoomKey(pub RoomID);
