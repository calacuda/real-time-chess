use bevy::prelude::*;
use bevy_renet::renet::{ChannelConfig, ClientId, ConnectionConfig, SendType};
use serde::{Deserialize, Serialize};
use std::{f32::consts::PI, time::Duration};

pub mod components;
pub mod events;
pub mod plugins;
pub mod states;
pub mod systems;

pub const PROTOCOL_ID: u64 = 7;

pub type Location = (Rank, File);
pub type RoomID = [char; 4];
pub type UserName = String;

pub fn display_room_id(id: &RoomID) -> String {
    format!("{}-{}-{}-{}", id[0], id[1], id[2], id[3])
}

#[derive(Debug, Clone, Copy, Component, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct Slope {
    pub rise: f32,
    pub run: f32,
}

impl Slope {
    pub fn to_degrees(&self) -> f32 {
        if self.run == 0.0 && self.rise > 0.0 {
            90.0
        } else if self.run == 0.0 && self.rise < 0.0 {
            270.0
        } else {
            (self.rise / self.run).atan() * 180.0 / PI
        }
    }
}

#[derive(Debug, Clone, Copy, Component, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Player {
    pub id: ClientId,
    pub color: PlayerColor,
    pub cooldown: Duration,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum File {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
}

impl Into<usize> for File {
    fn into(self) -> usize {
        match self {
            Self::One => 0,
            Self::Two => 1,
            Self::Three => 2,
            Self::Four => 3,
            Self::Five => 4,
            Self::Six => 5,
            Self::Seven => 6,
            Self::Eight => 7,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Rank {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

impl Into<usize> for Rank {
    fn into(self) -> usize {
        match self {
            Self::A => 0,
            Self::B => 1,
            Self::C => 2,
            Self::D => 3,
            Self::E => 4,
            Self::F => 5,
            Self::G => 6,
            Self::H => 7,
        }
    }
}

#[derive(Debug, Clone, Copy, Component, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChessPiece {
    K,
    Q,
    B,
    N,
    R,
    Pawn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientInGameMessage {
    Move {
        from: (Rank, File),
        to: (Rank, File),
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientSystemMessage {
    StartRoom(RoomID),
    JoinRoom(RoomID),
    ListRooms,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Resource)]
pub enum PlayerColor {
    Black,
    White,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cooldown {
    pos: Location,
    time_left: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerInRoomMessage {
    /// tells the client that the server is waiting for another player to join the game.
    WaitingForPlayers,
    /// player requested to join
    RoomJoinRequest(UserName),
    /// player successfully joined the room
    PlayerJoined(UserName),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerInGameMessage {
    /// the move was recieved and made successfully.
    MoveRecv {
        player: PlayerColor,
        from: Location,
        to: Location,
        capture: bool,
        cooldown: Duration,
    },
    /// the peice can't move like that caries the message which describes how/why that move
    /// was invalid.  
    InvalidMove(String),
    /// a player captured the king
    Victory(PlayerColor),
    Draw,
    OpponentDisconect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerSystemMessage {
    /// a list of the rooms available to join.
    ListRooms(Vec<RoomID>),
    /// Misc error.
    Error(String),
    /// notifies a client that they joined a room.
    JoinedRoom(RoomID),
    /// notifies a client that they left a room.
    LeftRoom(RoomID),
}

pub enum ClientChannel {
    Game,
    System,
}
pub enum ServerChannel {
    InGame,
    InRoom,
    System,
}

impl From<ClientChannel> for u8 {
    fn from(channel_id: ClientChannel) -> Self {
        match channel_id {
            ClientChannel::System => 0,
            ClientChannel::Game => 1,
        }
    }
}

impl ClientChannel {
    pub fn channels_config() -> Vec<ChannelConfig> {
        vec![
            ChannelConfig {
                channel_id: Self::System.into(),
                max_memory_usage_bytes: 5 * 1024 * 1024,
                send_type: SendType::ReliableOrdered {
                    resend_time: Duration::ZERO,
                },
            },
            ChannelConfig {
                channel_id: Self::Game.into(),
                max_memory_usage_bytes: 5 * 1024 * 1024,
                send_type: SendType::ReliableOrdered {
                    resend_time: Duration::ZERO,
                },
            },
        ]
    }
}

impl From<ServerChannel> for u8 {
    fn from(channel_id: ServerChannel) -> Self {
        match channel_id {
            ServerChannel::System => 0,
            ServerChannel::InGame => 1,
            ServerChannel::InRoom => 2,
        }
    }
}

impl ServerChannel {
    pub fn channels_config() -> Vec<ChannelConfig> {
        vec![
            ChannelConfig {
                channel_id: Self::System.into(),
                max_memory_usage_bytes: 10 * 1024 * 1024,
                send_type: SendType::Unreliable,
            },
            ChannelConfig {
                channel_id: Self::InGame.into(),
                max_memory_usage_bytes: 10 * 1024 * 1024,
                send_type: SendType::ReliableOrdered {
                    resend_time: Duration::from_millis(200),
                },
            },
            ChannelConfig {
                channel_id: Self::InRoom.into(),
                max_memory_usage_bytes: 10 * 1024 * 1024,
                send_type: SendType::Unreliable,
            },
        ]
    }
}

pub fn connection_config() -> ConnectionConfig {
    ConnectionConfig {
        available_bytes_per_tick: 1024 * 1024,
        client_channels_config: ClientChannel::channels_config(),
        server_channels_config: ServerChannel::channels_config(),
    }
}
