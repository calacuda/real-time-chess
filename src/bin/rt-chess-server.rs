#![feature(let_chains)]
use anyhow::{Result, ensure};
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::EguiPlugin;
use bevy_renet::{
    RenetServerPlugin,
    renet::{ClientId, RenetServer, ServerEvent},
};
use real_time_chess::{
    ChessPiece, ClientChannel, ClientInGameMessage, ClientSystemMessage, File, Location,
    PROTOCOL_ID, Player, PlayerColor, Rank, RoomID, ServerChannel, ServerInGameMessage,
    ServerSystemMessage, Slope, connection_config,
};
use renet_visualizer::RenetServerVisualizer;
use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
    time::{Duration, Instant},
};

pub mod server;

pub type BoardPiece = (ChessPiece, PlayerColor, Instant, Duration);
pub type BoardSquare = Option<BoardPiece>;

#[derive(Debug, Clone)]
pub struct Board {
    squares: [[BoardSquare; 8]; 8],
}

impl Default for Board {
    fn default() -> Self {
        let mut room = Board {
            squares: [[None; 8]; 8],
        };
        let white_pieces: Vec<(Location, ChessPiece)> = vec![
            ((Rank::A, File::One), ChessPiece::R),
            ((Rank::B, File::One), ChessPiece::N),
            ((Rank::C, File::One), ChessPiece::B),
            ((Rank::D, File::One), ChessPiece::Q),
            ((Rank::E, File::One), ChessPiece::K),
            ((Rank::F, File::One), ChessPiece::B),
            ((Rank::G, File::One), ChessPiece::N),
            ((Rank::H, File::One), ChessPiece::R),
            ((Rank::A, File::Two), ChessPiece::Pawn),
            ((Rank::B, File::Two), ChessPiece::Pawn),
            ((Rank::C, File::Two), ChessPiece::Pawn),
            ((Rank::D, File::Two), ChessPiece::Pawn),
            ((Rank::E, File::Two), ChessPiece::Pawn),
            ((Rank::F, File::Two), ChessPiece::Pawn),
            ((Rank::G, File::Two), ChessPiece::Pawn),
            ((Rank::H, File::Two), ChessPiece::Pawn),
        ];
        let black_pieces: Vec<(Location, ChessPiece)> = vec![
            ((Rank::A, File::Eight), ChessPiece::R),
            ((Rank::B, File::Eight), ChessPiece::N),
            ((Rank::C, File::Eight), ChessPiece::B),
            ((Rank::D, File::Eight), ChessPiece::Q),
            ((Rank::E, File::Eight), ChessPiece::K),
            ((Rank::F, File::Eight), ChessPiece::B),
            ((Rank::G, File::Eight), ChessPiece::N),
            ((Rank::H, File::Eight), ChessPiece::R),
            ((Rank::A, File::Seven), ChessPiece::Pawn),
            ((Rank::B, File::Seven), ChessPiece::Pawn),
            ((Rank::C, File::Seven), ChessPiece::Pawn),
            ((Rank::D, File::Seven), ChessPiece::Pawn),
            ((Rank::E, File::Seven), ChessPiece::Pawn),
            ((Rank::F, File::Seven), ChessPiece::Pawn),
            ((Rank::G, File::Seven), ChessPiece::Pawn),
            ((Rank::H, File::Seven), ChessPiece::Pawn),
        ];

        let inst = Instant::now();
        let dur = Duration::from_secs_f32(0.0);

        for (loc, piece) in white_pieces {
            room[&loc] = Some((piece, PlayerColor::White, inst, dur));
        }

        for (loc, piece) in black_pieces {
            room[&loc] = Some((piece, PlayerColor::Black, inst, dur));
        }

        room
    }
}

impl Board {
    pub fn get_coords(&self) -> Vec<((usize, usize), BoardPiece)> {
        let all_squares = self.squares.iter().enumerate().map(move |(rank, files)| {
            files
                .iter()
                .enumerate()
                .map(move |(file, squares)| squares.map(|square| ((rank, file), square)))
        });
        all_squares.flatten().filter_map(|square| square).collect()
    }
}

impl Index<&Location> for Board {
    type Output = Option<(ChessPiece, PlayerColor, Instant, Duration)>;

    fn index(&self, index: &Location) -> &Self::Output {
        let rank: usize = index.0.into();
        let f: usize = index.1.into();
        &self.squares[rank][f]
    }
}

impl IndexMut<&Location> for Board {
    fn index_mut(&mut self, index: &Location) -> &mut Self::Output {
        let rank: usize = index.0.into();
        let f: usize = index.1.into();
        &mut self.squares[rank][f]
    }
}

#[derive(Debug, Default, Clone, Component)]
pub struct Room {
    id: RoomID,
    board: Board,
    // TODO: add a Host and player2 field
    // TODO: add a spectators field
}

impl Room {
    pub fn make_move_for(
        &mut self,
        player: &mut Player,
        from: Location,
        to: Location,
    ) -> ServerInGameMessage {
        let Some((piece, moving_peice_color, last_moved, cooldown)) = self.board[&from].clone()
        else {
            error!("{}, tried to move a nonexisting peice.", player.id);
            return ServerInGameMessage::InvalidMove(format!(
                "there is no peice to move at possision {from:?}"
            ));
        };

        if !self.moving_own_peice(player.color, moving_peice_color) {
            return ServerInGameMessage::InvalidMove(format!("you can only move your own peices."));
        }

        if self.self_capture(player.color, &to) {
            return ServerInGameMessage::InvalidMove(format!(
                "you can't move a peice to a square ocupied by one of your own peice."
            ));
        }

        // calculate a vector of movement for the peice and see if its valid.
        if let Err(e) = self.validated_move_vec(piece, &from, &to, moving_peice_color) {
            return ServerInGameMessage::InvalidMove(format!("{e}"));
        }

        if self.durring_cooldown(last_moved, cooldown) {
            player.cooldown += cooldown / 3;

            return ServerInGameMessage::InvalidMove(format!(
                "peice at possision {from:?} is on cooldown."
            ));
        } else if self.penalty_move(last_moved, cooldown) {
            player.cooldown += cooldown / 4;
        }

        // check for capture.
        let capture = self.capture(player.color, &to);

        // move peice
        self.make_move(&from, &to, player.cooldown);

        ServerInGameMessage::MoveRecv {
            player: player.color,
            from,
            to,
            capture,
            cooldown: player.cooldown,
        }
    }

    fn validated_move_vec(
        &mut self,
        piece: ChessPiece,
        from: &Location,
        to: &Location,
        moving_peice_color: PlayerColor,
    ) -> Result<()> {
        let move_vector = self.calc_move_vec(&from, &to);
        let (magnitude, slope) = move_vector;

        if piece != ChessPiece::N {
            self.piece_in_way(magnitude, slope, from, moving_peice_color)?;
        }

        let angle = slope.to_degrees();

        match piece {
            ChessPiece::K => self.validate_king_move(magnitude, angle),
            ChessPiece::N => self.validate_knight_move(magnitude, angle),
            ChessPiece::Q => self.validate_queen_move(angle),
            ChessPiece::B => self.validate_bishop_move(angle),
            ChessPiece::R => self.validate_rook_move(angle),
            ChessPiece::Pawn => {
                self.validate_pawn_move(magnitude, angle, from, to, moving_peice_color)
            }
        }?;

        Ok(())
    }

    fn piece_in_way(
        &mut self,
        magnitude: f32,
        angle: Slope,
        from: &Location,
        // to: &Location,
        moving_peice_color: PlayerColor,
    ) -> Result<()> {
        let make_key = |angle: Slope| format!("({:.2}/{:.2})", angle.rise, angle.run);

        let piece_vecs: HashMap<String, (f32, PlayerColor)> = self
            .board
            .get_coords()
            .into_iter()
            .map(|((rank, file), (_, color, _, _))| {
                let (mag, angle) = self.do_calc_move_vec(
                    (from.0 as usize) as f32,
                    rank as f32,
                    (from.1 as usize) as f32,
                    file as f32,
                );
                (make_key(angle), (mag, color))
            })
            .collect();

        let same_vec = piece_vecs.get(&make_key(angle));

        ensure!(
            same_vec.is_some_and(|(mag, color)| {
                (moving_peice_color == *color && *mag < magnitude)
                    || (moving_peice_color != *color && *mag <= magnitude)
            }) || same_vec.is_none(),
            "there was a piece in the way of that movement"
        );

        Ok(())
    }

    fn validate_king_move(
        &mut self,
        magnitude: f32,
        angle: f32,
        // from: &Location,
        // to: &Location,
    ) -> Result<()> {
        ensure!(
            ((magnitude == 1.414 && vec![45., 135., 225., 315.].contains(&angle))
                || (magnitude == 1. && vec![0.0, 90., 180., 270.].contains(&angle))),
        );

        Ok(())
    }

    fn validate_knight_move(
        &mut self,
        magnitude: f32,
        angle: f32,
        // from: &Location,
        // to: &Location,
    ) -> Result<()> {
        ensure!(
            vec![1. / 2., 2. / 1.].contains(&angle.abs()),
            "Knight movement had wrong angle."
        );

        // check magnitude is 2.23...
        let mag = (magnitude * 100.).round() / 100.0;
        ensure!(mag == 2.24, "Knight movement had wrong magnitude.");

        Ok(())
    }

    fn validate_queen_move(
        &mut self,
        // magnitude: f32,
        angle: f32,
        // from: &Location,
        // to: &Location,
    ) -> Result<()> {
        ensure!(
            vec![45., 135., 225., 315., 0.0, 90., 180., 270.].contains(&angle),
            "the queen must move diaganoly, vertically, or horzonatally."
        );

        Ok(())
    }

    fn validate_bishop_move(
        &mut self,
        // magnitude: f32,
        angle: f32,
        // from: &Location,
        // to: &Location,
    ) -> Result<()> {
        ensure!(
            vec![45., 135., 225., 315.].contains(&angle),
            "the bishop can only move diagonally."
        );

        Ok(())
    }

    fn validate_rook_move(
        &mut self,
        // magnitude: f32,
        angle: f32,
        // from: &Location,
        // to: &Location,
    ) -> Result<()> {
        ensure!(
            vec![0.0, 90., 180., 270.].contains(&angle),
            "the rook can only move vertically or horizontally."
        );

        Ok(())
    }

    fn validate_pawn_move(
        &mut self,
        magnitude: f32,
        angle: f32,
        from: &Location,
        to: &Location,
        moving_peice_color: PlayerColor,
    ) -> Result<()> {
        // TODO: include en pesant
        if (2 == from.0 as usize
            && self.board[from].is_some_and(
                |(_piece, moving_peice_color, _last_moved, _cooldown)| {
                    moving_peice_color == PlayerColor::White
                },
            )
            && angle == 90.
            && magnitude == 2.)
            || (7 == from.0 as usize
                && self.board[from].is_some_and(
                    |(_piece, moving_peice_color, _last_moved, _cooldown)| {
                        moving_peice_color == PlayerColor::Black
                    },
                )
                && angle == 270.
                && magnitude == 2.)
        {
            return Ok(());
        }

        ensure!(
            angle == 45.0 || angle == 135.0 || angle == 90.0,
            "pawns can only go forward."
        );

        if angle == 45. || angle == 135. {
            ensure!(
                self.board[to].is_some_and(|(_, color, _, _)| moving_peice_color != color)
                    && ((magnitude * 100.).round() / 100.) == 1.41,
                "pawns can only move diaganoly when capturing a piece."
            );
        } else if angle == 90. {
            ensure!(
                self.board[to].is_none() && magnitude == 1.,
                "pawns cannot capture forwards."
            );
        }

        Ok(())
    }

    pub fn do_calc_move_vec(&mut self, x1: f32, x2: f32, y1: f32, y2: f32) -> (f32, Slope) {
        let magnitude = ((x1 - x2).powi(2) + (y1 - y2).powi(2)).sqrt();
        let angle = Slope {
            rise: (y1 - y2),
            run: (x1 - x2),
        };

        (magnitude, angle)
    }

    pub fn calc_move_vec(&mut self, from: &Location, to: &Location) -> (f32, Slope) {
        let x1 = (from.0 as usize) as f32;
        let x2 = (to.0 as usize) as f32;

        let y1 = (from.1 as usize) as f32;
        let y2 = (to.1 as usize) as f32;

        self.do_calc_move_vec(x1, x2, y1, y2)
    }

    pub fn make_move(&mut self, from: &Location, to: &Location, cooldown: Duration) {
        let piece =
            self.board[&from]
                .clone()
                .map(|(piece, moving_peice_color, _last_moved, _cooldown)| {
                    (
                        piece,
                        moving_peice_color,
                        Instant::now() + Duration::from_secs_f32(0.1),
                        cooldown,
                    )
                });
        self.board[&to] = piece;
        self.board[&from] = None;
    }

    pub fn penalty_move(&mut self, last_moved: Instant, cooldown: Duration) -> bool {
        last_moved.elapsed() > cooldown / 4 * 3 && last_moved.elapsed() < cooldown
    }

    pub fn durring_cooldown(&mut self, last_moved: Instant, cooldown: Duration) -> bool {
        last_moved.elapsed() < cooldown / 4 * 3
    }

    pub fn moving_own_peice(
        &mut self,
        player_color: PlayerColor,
        moving_peice_color: PlayerColor,
    ) -> bool {
        moving_peice_color == player_color
    }

    pub fn capture(&mut self, player_color: PlayerColor, to: &Location) -> bool {
        self.board[&to]
            .clone()
            .is_some_and(|(_, color, _, _)| color != player_color)
    }

    pub fn self_capture(&mut self, player_color: PlayerColor, to: &Location) -> bool {
        self.board[&to]
            .clone()
            .is_some_and(|(_, color, _, _)| color == player_color)
    }
}

#[derive(Debug, Default, Resource)]
pub struct ServerLobby {
    pub players: HashMap<ClientId, Player>,
    pub room_mem: HashMap<ClientId, RoomID>,
}

fn add_network(app: &mut App) {
    use bevy_renet::netcode::{
        NetcodeServerPlugin, NetcodeServerTransport, ServerAuthentication, ServerConfig,
    };
    // use demo_bevy:PROTOCOL_ID, connection_config};
    use std::{net::UdpSocket, time::SystemTime};

    app.add_plugins(NetcodeServerPlugin);

    let server = RenetServer::new(connection_config());

    let public_addr = "127.0.0.1:5000".parse().unwrap();
    let socket = UdpSocket::bind(public_addr).unwrap();
    let current_time: std::time::Duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let server_config = ServerConfig {
        current_time,
        max_clients: 64,
        protocol_id: PROTOCOL_ID,
        public_addresses: vec![public_addr],
        authentication: ServerAuthentication::Unsecure,
    };

    let transport = NetcodeServerTransport::new(server_config, socket).unwrap();
    app.insert_resource(server);
    app.insert_resource(transport);
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);

    app.add_plugins(RenetServerPlugin);
    app.add_plugins(FrameTimeDiagnosticsPlugin);
    app.add_plugins(LogDiagnosticsPlugin::default());
    app.add_plugins(EguiPlugin);

    app.insert_resource(ServerLobby::default());

    app.insert_resource(RenetServerVisualizer::<200>::default());

    add_network(&mut app);

    app.add_systems(Update, server_update_system);

    // app.add_systems(FixedUpdate, apply_velocity_system);
    // app.add_systems(PostUpdate, projectile_on_removal_system);
    // app.add_systems(Startup, (setup_level, setup_simple_camera));

    app.run();
}

fn server_update_system(
    mut server_events: EventReader<ServerEvent>,
    mut commands: Commands,
    mut lobby: ResMut<ServerLobby>,
    mut rooms: Query<&mut Room>,
    mut server: ResMut<RenetServer>,
    mut visualizer: ResMut<RenetServerVisualizer<200>>,
) {
    for event in server_events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                info!("Player {} connected.", client_id);
                visualizer.add_client(*client_id);

                lobby.players.insert(*client_id, Player {
                    id: *client_id,
                    color: PlayerColor::White,
                    cooldown: Duration::from_secs(5),
                });
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                info!("Player {} disconnected: {}", client_id, reason);
                visualizer.remove_client(*client_id);
                if lobby.players.remove(client_id).is_some() {
                    lobby.room_mem.remove(&client_id);
                }
            }
        }
    }

    for client_id in server.clients_id() {
        while let Some(message) = server.receive_message(client_id, ClientChannel::System) {
            if let Ok(command) = bincode::deserialize::<ClientSystemMessage>(&message) {
                match command {
                    ClientSystemMessage::ListRooms => {
                        let message = ServerSystemMessage::ListRooms(
                            // lobby.rooms.clone().keys().map(|key| *key).collect(),
                            rooms.iter().map(|room| room.id.clone()).collect(),
                        );
                        let message = bincode::serialize(&message).unwrap();
                        server.send_message(client_id, ServerChannel::System, message);
                    }
                    // ClientMessage::ChatMessage(_mesg) => {}
                    ClientSystemMessage::StartRoom(room_key) => {
                        if lobby.players.get(&client_id).is_some()
                            && !lobby.room_mem.contains_key(&client_id)
                        {
                            // lobby.rooms.insert(room_key, Room::default());
                            if rooms.iter().position(|room| room.id == room_key).is_none() {
                                commands.spawn(Room {
                                    id: room_key,
                                    board: Board::default(),
                                });
                                lobby.room_mem.insert(client_id, room_key);
                                let msg = ServerSystemMessage::JoinedRoom(room_key);
                                server.send_message(
                                    client_id,
                                    ServerChannel::System,
                                    bincode::serialize(&msg).unwrap(),
                                );
                            } else {
                                let msg = ServerSystemMessage::Error(
                                    "that room key already exists. try a different one.".into(),
                                );
                                server.send_message(
                                    client_id,
                                    ServerChannel::System,
                                    bincode::serialize(&msg).unwrap(),
                                );
                            }
                        } else {
                            let msg = ServerSystemMessage::Error("you are already in a room. please leave the room before trying to start a new one.".into());
                            server.send_message(
                                client_id,
                                ServerChannel::System,
                                bincode::serialize(&msg).unwrap(),
                            );
                        }
                    }
                    ClientSystemMessage::JoinRoom(room_key) => {
                        // TODO: add room join requesting and the like.
                        let room_exists =
                            rooms.iter().position(|room| room.id == room_key).is_some();
                        if room_exists && !lobby.room_mem.contains_key(&client_id) {
                            lobby.room_mem.remove(&client_id);
                            lobby.room_mem.insert(client_id, room_key);
                        } else {
                            let message = bincode::serialize(&if !room_exists {
                                ServerSystemMessage::Error("that room doen't exist".into())
                            } else if lobby.room_mem.contains_key(&client_id) {
                                ServerSystemMessage::Error("you're already in a room".into())
                            } else {
                                ServerSystemMessage::Error("can't join that room right now.".into())
                            });
                            if let Ok(message) = message {
                                server.send_message(client_id, ServerChannel::System, message);
                            } else {
                                error!("could not serialize JoinRoomFailure message.");
                            }
                        }
                    }
                }
            }
        }

        while let Some(message) = server.receive_message(client_id, ClientChannel::Game) {
            if let Ok(command) = bincode::deserialize::<ClientInGameMessage>(&message) {
                match command {
                    ClientInGameMessage::Move { from, to } => {
                        let room_mem = lobby.room_mem.clone();
                        let room = room_mem.get(&client_id);

                        if let Some(room_id) = room {
                            if let Some(ref mut player) = lobby.players.get_mut(&client_id) {
                                rooms.iter_mut().for_each(|mut room| {
                                    if room.id == room_id.clone() {
                                        let message = bincode::serialize(
                                            &room.make_move_for(player, from, to),
                                        )
                                        .unwrap();
                                        server.send_message(
                                            client_id,
                                            ServerChannel::InGame,
                                            message,
                                        );

                                        // TODO: check for victory.
                                        // TODO: check for promotion.
                                    }
                                });
                            } else {
                                let message = bincode::serialize(&ServerSystemMessage::Error(
                                    "that room has closed. the game ended.".into(),
                                ))
                                .unwrap();
                                server.send_message(client_id, ServerChannel::System, message);
                            }
                        } else {
                            let message = bincode::serialize(&ServerSystemMessage::Error(
                                "you're not in a room. join/start on first".into(),
                            ))
                            .unwrap();
                            server.send_message(client_id, ServerChannel::System, message);
                        }
                    }
                }
            }
        }
    }
}
