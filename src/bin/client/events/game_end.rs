use bevy::prelude::*;

#[derive(Debug, Clone, Event)]
pub enum GameEnd {
    Victory,
    Loss,
    Draw,
    OpponentDisconnect,
}
