use bevy::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Event)]
pub struct InvalidMoveNotif {
    pub message: String,
}
