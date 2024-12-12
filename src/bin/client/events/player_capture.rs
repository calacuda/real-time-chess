use bevy::prelude::*;
use real_time_chess::{ChessPiece, Location};

#[derive(Debug, Clone, Event)]
pub struct PlayerCaptureNotif {
    pub piece: ChessPiece,
    pub square: Location,
}
