use bevy::prelude::*;

#[derive(States, Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GameState {
    InGame,
    InRoom,
    #[default]
    Startup,
    RoomSelect,
    StartNewRoom,
    Settings,
}
