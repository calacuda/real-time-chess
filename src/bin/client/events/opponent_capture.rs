use bevy::prelude::*;
use real_time_chess::{ChessPiece, Location};

#[derive(Debug, Clone, Event)]
pub struct OpponentCaptureNotif {
    piece: ChessPiece,
    square: Location,
}
