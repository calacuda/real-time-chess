#![feature(let_chains)]
use anyhow::{Result, ensure};
use async_trait::async_trait;
use real_time_chess::{
    ChessPiece, File, Location, Player, PlayerColor, Rank, RoomID, ServerInGameMessage, Slope,
};
use russh::{
    Channel, ChannelId, CryptoVec, Preferred,
    keys::Certificate,
    server::{self, Msg, Server as _, Session},
};
use russh_keys::{PublicKey, ssh_key::rand_core::OsRng};
use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;
use tracing::*;

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

#[derive(Debug, Default, Clone)]
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
            error!("{}, tried to move a nonexisting peice.", player.user_name);
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

#[derive(Clone)]
struct Server {
    clients: Arc<Mutex<HashMap<usize, (ChannelId, russh::server::Handle)>>>,
    id: usize,
}

impl Server {
    async fn post(&mut self, data: CryptoVec) {
        let mut clients = self.clients.lock().await;
        for (id, &mut (ref mut channel, ref mut s)) in clients.iter_mut() {
            if *id != self.id {
                let _ = s.data(*channel, data.clone()).await;
            }
        }
    }
}

impl server::Server for Server {
    type Handler = Self;
    fn new_client(&mut self, _: Option<std::net::SocketAddr>) -> Self {
        let s = self.clone();
        self.id += 1;
        s
    }
    fn handle_session_error(&mut self, _error: <Self::Handler as russh::server::Handler>::Error) {
        eprintln!("Session error: {:#?}", _error);
    }
}

#[async_trait]
impl server::Handler for Server {
    type Error = russh::Error;

    async fn channel_open_session(
        &mut self,
        channel: Channel<Msg>,
        session: &mut Session,
    ) -> Result<bool, Self::Error> {
        {
            let mut clients = self.clients.lock().await;
            clients.insert(self.id, (channel.id(), session.handle()));
        }
        Ok(true)
    }

    async fn auth_publickey(
        &mut self,
        _: &str,
        _key: &PublicKey,
    ) -> Result<server::Auth, Self::Error> {
        Ok(server::Auth::Accept)
    }

    async fn auth_openssh_certificate(
        &mut self,
        _user: &str,
        certificate: &Certificate,
    ) -> Result<server::Auth, Self::Error> {
        dbg!(certificate);
        Ok(server::Auth::Accept)
    }

    async fn data(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        // Sending Ctrl+C ends the session and disconnects the client
        if data == [3] {
            return Err(russh::Error::Disconnect);
        }

        let data = CryptoVec::from(format!("Got data: {}\r\n", String::from_utf8_lossy(data)));
        self.post(data.clone()).await;
        session.data(channel, data)?;
        Ok(())
    }

    // async fn tcpip_forward(
    //     &mut self,
    //     address: &str,
    //     port: &mut u32,
    //     session: &mut Session,
    // ) -> Result<bool, Self::Error> {
    //     let handle = session.handle();
    //     let address = address.to_string();
    //     let port = *port;
    //     tokio::spawn(async move {
    //         let channel = handle
    //             .channel_open_forwarded_tcpip(address, port, "1.2.3.4", 1234)
    //             .await
    //             .unwrap();
    //         let _ = channel.data(&b"Hello from a forwarded port"[..]).await;
    //         let _ = channel.eof().await;
    //     });
    //     Ok(true)
    // }
}

impl Drop for Server {
    fn drop(&mut self) {
        let id = self.id;
        let clients = self.clients.clone();
        tokio::spawn(async move {
            let mut clients = clients.lock().await;
            clients.remove(&id);
        });
    }
}

#[tokio::main]
async fn main() {
    let config = russh::server::Config {
        inactivity_timeout: Some(std::time::Duration::from_secs(3600)),
        auth_rejection_time: std::time::Duration::from_secs(3),
        auth_rejection_time_initial: Some(std::time::Duration::from_secs(0)),
        keys: vec![
            russh_keys::PrivateKey::random(&mut OsRng, russh_keys::Algorithm::Ed25519).unwrap(),
        ],
        preferred: Preferred::default(),
        ..Default::default()
    };
    let config = Arc::new(config);
    let mut sh = Server {
        clients: Arc::new(Mutex::new(HashMap::new())),
        id: 0,
    };
    sh.run_on_address(config, ("127.0.0.1", 2222))
        .await
        .unwrap();
}
