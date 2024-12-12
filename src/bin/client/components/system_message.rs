use bevy::prelude::*;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SystemMessageType {
    InvalidMove,
    MiscError,
    Alert,
    RoomJoin,
    RoomLeave,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Component)]
pub struct SystemMessage {
    pub message: String,
    pub msg_type: SystemMessageType,
    pub display_duration: Duration,
    pub shown: Option<Instant>,
}

// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Resource)]
// pub struct SystemMessages(Vec<SystemMessage>);
