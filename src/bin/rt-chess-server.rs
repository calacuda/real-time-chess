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
    ChessPiece, ClientChannel, ClientMessage, Location, PROTOCOL_ID, Player, PlayerColor, RoomID,
    ServerChannel, ServerMessage, Slope, connection_config,
};
use renet_visualizer::RenetServerVisualizer;
use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
    time::{Duration, Instant},
};

pub type BoardPiece = (ChessPiece, PlayerColor, Instant, Duration);
pub type BoardSquare = Option<BoardPiece>;

#[derive(Debug, Default, Clone)]
pub struct Board {
    squares: [[BoardSquare; 8]; 8],
}

impl Board {
    pub fn get_cords(&self) -> Vec<((usize, usize), BoardPiece)> {
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
}

impl Room {
    pub fn make_move_for(
        &mut self,
        player: &mut Player,
        from: Location,
        to: Location,
    ) -> ServerMessage {
        let Some((piece, moving_peice_color, last_moved, cooldown)) = self.board[&from].clone()
        else {
            error!("{}, tried to move a nonexisting peice.", player.id);
            return ServerMessage::InvalidMove(format!(
                "there is no peice to move at possision {from:?}"
            ));
        };

        if !self.moving_own_peice(player.color, moving_peice_color) {
            return ServerMessage::InvalidMove(format!("you can only move your own peices."));
        }

        if self.self_capture(player.color, &to) {
            return ServerMessage::InvalidMove(format!(
                "you can't move a peice to a square ocupied by one of your own peice."
            ));
        }

        // calculate a vector of movement for the peice and see if its valid.
        if let Err(e) = self.validated_move_vec(piece, &from, &to) {
            return ServerMessage::InvalidMove(format!("{e}"));
        }

        if self.durring_cooldown(last_moved, cooldown) {
            player.cooldown += cooldown / 3;

            return ServerMessage::InvalidMove(format!(
                "peice at possision {from:?} is on cooldown."
            ));
        } else if self.penalty_move(last_moved, cooldown) {
            player.cooldown += cooldown / 4;
        }

        // check for capture.
        let capture = self.capture(player.color, &from);

        // move peice
        self.make_move(&from, &to, player.cooldown);

        ServerMessage::MoveRecv {
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
    ) -> Result<()> {
        let move_vector = self.calc_move_vec(&from, &to);
        let (magnitude, slope) = move_vector;

        if piece != ChessPiece::N {
            self.piece_in_way(magnitude, slope, from)?;
        }

        let angle = slope.to_degrees();

        match piece {
            ChessPiece::K => self.validate_king_move(magnitude, angle),
            ChessPiece::N => self.validate_knight_move(magnitude, angle),
            ChessPiece::Q => self.validate_queen_move(angle),
            ChessPiece::B => self.validate_bishop_move(angle),
            ChessPiece::R => self.validate_rook_move(angle),
            ChessPiece::Pawn => self.validate_pawn_move(magnitude, angle, from, to),
        }?;

        Ok(())
    }

    fn piece_in_way(
        &mut self,
        magnitude: f32,
        angle: Slope,
        from: &Location,
        // to: &Location,
    ) -> Result<()> {
        // let mut piece_vecs = HashMap::with_capacity(16);
        let make_key = |angle: Slope| format!("({:.2}/{:.2})", angle.rise, angle.run);

        let piece_vecs: HashMap<String, f32> = self
            .board
            .get_cords()
            .into_iter()
            .map(|((rank, file), _)| {
                let (mag, angle) = self.do_calc_move_vec(
                    (from.0 as usize) as f32,
                    rank as f32,
                    (from.1 as usize) as f32,
                    file as f32,
                );
                (make_key(angle), mag)
            })
            .collect();

        if let Some(mag) = piece_vecs.get(&make_key(angle)) {
            ensure!(
                *mag >= magnitude,
                "there was a piece in the way of that movement"
            )
        }

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
                self.board[to].is_some() && ((magnitude * 100.).round() / 100.) == 1.41,
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

    pub fn capture(&mut self, player_color: PlayerColor, from: &Location) -> bool {
        self.board[&from]
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
        while let Some(message) = server.receive_message(client_id, ClientChannel::Command) {
            if let Ok(command) = bincode::deserialize::<ClientMessage>(&message) {
                match command {
                    ClientMessage::ListRooms => {
                        let message = ServerMessage::ListRooms(
                            // lobby.rooms.clone().keys().map(|key| *key).collect(),
                            rooms.iter().map(|room| room.id.clone()).collect(),
                        );
                        let message = bincode::serialize(&message).unwrap();
                        server.broadcast_message(ServerChannel::ServerMessages, message);
                    }
                    // ClientMessage::ChatMessage(_mesg) => {}
                    ClientMessage::StartRoom(room_key) => {
                        if lobby.players.get(&client_id).is_some()
                            && !lobby.room_mem.contains_key(&client_id)
                        {
                            // lobby.rooms.insert(room_key, Room::default());
                            commands.spawn(Room {
                                id: room_key,
                                board: Board::default(),
                            });
                            lobby.room_mem.insert(client_id, room_key);
                        }
                    }
                    ClientMessage::JoinRoom(room_key) => {
                        let room_exists =
                            rooms.iter().position(|room| room.id == room_key).is_some();
                        if room_exists && !lobby.room_mem.contains_key(&client_id) {
                            lobby.room_mem.remove(&client_id);
                            lobby.room_mem.insert(client_id, room_key);
                        } else {
                            let message = bincode::serialize(&if !room_exists {
                                ServerMessage::Error("that room doen't exist".into())
                            } else if lobby.room_mem.contains_key(&client_id) {
                                ServerMessage::Error("you're already in a room".into())
                            } else {
                                ServerMessage::Error("can't join that room right now.".into())
                            });
                            if let Ok(message) = message {
                                server.broadcast_message(ServerChannel::ServerMessages, message);
                            } else {
                                error!("could not serialize JoinRoomFailure message.");
                            }
                        }
                    }
                    ClientMessage::Move { from, to } => {
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
                                        server.broadcast_message(
                                            ServerChannel::ServerMessages,
                                            message,
                                        );

                                        // TODO: check for victory.
                                        // TODO: check for promotion.
                                    }
                                });
                            } else {
                                let message = bincode::serialize(&ServerMessage::Error(
                                    "that room has closed. the game ended.".into(),
                                ))
                                .unwrap();
                                server.broadcast_message(ServerChannel::ServerMessages, message);
                            }
                        } else {
                            let message = bincode::serialize(&ServerMessage::Error(
                                "you're not in a room. join/start on first".into(),
                            ))
                            .unwrap();
                            server.broadcast_message(ServerChannel::ServerMessages, message);
                        }
                    }
                }
            }
        }
    }
}
