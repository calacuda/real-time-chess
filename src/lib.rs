use bevy::prelude::*;
use bevy_renet::renet::{ChannelConfig, ClientId, SendType};
use serde::{Deserialize, Serialize};
use std::{f32::consts::PI, time::Duration};

pub type Location = (Rank, File);
pub type RoomID = [char; 4];

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
pub enum ClientMessage {
    ChatMessage(String),
    Move {
        from: (Rank, File),
        to: (Rank, File),
    },
    StartRoom(RoomID),
    JoinRoom(RoomID),
    ListRooms,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
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
pub enum ServerMessage {
    /// tells the client that the server is looking for another player to match them against holds
    /// the duration before a "NoOponentFoundError"
    LoadingMatch(Duration),
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
    /// Chat message.
    ChatMessage(String),
    /// a player captured the king
    Victory(PlayerColor),
    /// updates on the cooldown of peices
    CooldownUpdate(Vec<Cooldown>),
    ///
    ListRooms(Vec<RoomID>),
    Error(String),
}

pub enum ClientChannel {
    // Input,
    Command,
}
pub enum ServerChannel {
    ServerMessages,
    // NetworkedEntities,
}

impl From<ClientChannel> for u8 {
    fn from(channel_id: ClientChannel) -> Self {
        match channel_id {
            ClientChannel::Command => 0,
            // ClientChannel::Input => 1,
        }
    }
}

impl ClientChannel {
    pub fn channels_config() -> Vec<ChannelConfig> {
        vec![
            // ChannelConfig {
            //     channel_id: Self::Input.into(),
            //     max_memory_usage_bytes: 5 * 1024 * 1024,
            //     send_type: SendType::ReliableOrdered {
            //         resend_time: Duration::ZERO,
            //     },
            // },
            ChannelConfig {
                channel_id: Self::Command.into(),
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
            // ServerChannel::NetworkedEntities => 0,
            ServerChannel::ServerMessages => 0,
        }
    }
}

impl ServerChannel {
    pub fn channels_config() -> Vec<ChannelConfig> {
        vec![
            // ChannelConfig {
            //     channel_id: Self::NetworkedEntities.into(),
            //     max_memory_usage_bytes: 10 * 1024 * 1024,
            //     send_type: SendType::Unreliable,
            // },
            ChannelConfig {
                channel_id: Self::ServerMessages.into(),
                max_memory_usage_bytes: 10 * 1024 * 1024,
                send_type: SendType::ReliableOrdered {
                    resend_time: Duration::from_millis(200),
                },
            },
        ]
    }
}
