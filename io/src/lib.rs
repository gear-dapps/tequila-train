#![no_std]

use gmeta::{InOut, Metadata};
use gstd::{prelude::*, ActorId};

#[derive(Encode, Decode, TypeInfo, Hash, PartialEq, PartialOrd, Eq, Ord, Clone, Copy, Debug)]
pub enum PingPong {
    Ping,
    Pong,
}

pub struct ContractMetadata;

impl Metadata for ContractMetadata {
    type Init = ();
    type Handle = InOut<PingPong, PingPong>;
    type Others = ();
    type Reply = ();
    type Signal = ();
    type State = State;
}

#[derive(Encode, Decode, TypeInfo, Hash, PartialEq, PartialOrd, Eq, Ord, Clone, Debug, Default)]
pub struct State(pub Vec<(ActorId, u128)>);

#[doc(hidden)]
impl State {
    pub fn pingers(self) -> Vec<ActorId> {
        self.0.into_iter().map(|pingers| pingers.0).collect()
    }

    pub fn ping_count(self, actor: ActorId) -> u128 {
        self.0
            .into_iter()
            .find_map(|(pinger, ping_count)| (pinger == actor).then_some(ping_count))
            .unwrap_or_default()
    }
}

#[derive(Encode, Decode, TypeInfo, Hash, PartialEq, PartialOrd, Eq, Ord, Clone, Copy, Debug)]
pub enum StateQuery {
    AllState,
    Pingers,
    PingCount(ActorId),
}

#[derive(Encode, Decode, TypeInfo, Hash, PartialEq, PartialOrd, Eq, Ord, Clone, Debug)]
pub enum StateQueryReply {
    AllState(<ContractMetadata as Metadata>::State),
    Pingers(Vec<ActorId>),
    PingCount(u128),
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, enum_iterator::Sequence)]
pub enum Face {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Eleven,
    Twelve,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Tile {
    first: Face,
    second: Face,
}

impl Tile {
    pub fn new(first: Face, second: Face) -> Self {
        Self { first, second }
    }

    pub fn is_double(&self) -> bool {
        self.first == self.second
    }

    pub fn can_be_stacked(&self, face: Face) -> Option<Face> {
        if self.first == face {
            return Some(self.second);
        }

        (self.second == face).then_some(self.first)
    }
}

pub fn build_tile_collection() -> Vec<Tile> {
    enum_iterator::all::<Face>()
        .enumerate()
        .map(|(i, face_first)| {
            enum_iterator::all::<Face>()
                .skip(i)
                .map(move |face_second| Tile::new(face_first, face_second))
        })
        .flatten()
        .collect()
}

pub struct Players {
    first: ActorId,
    second: ActorId,
    others: Vec<ActorId>,
}

// Type to identify the tile
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
struct TileId(Face, Face);

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
// A number to identify one of trace
struct TraceId(u32);

// Struct of payload to send move
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum GameMove {
    // Try to set a tile to a gameboard
    SetTile {
        tile_id: TileId,
        trace_id: TraceId,
        getting_train_back: bool,
    },
    // Try to set a train to your trace
    SetTrain {
        trace_id: TraceId,
    },
    // Skip the move and get new tile
    Skip,
    // Surrender and stop the game
    Surrender,
}

// Status to identify was the move correct or not
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
enum ReplyStatus {
    // Move correct
    Success,
    // User have no tile that is tryed to be set
    ErrorTileNotFound,
    // Invalid move turn
    ErrorNotYourTurnToMove,
    // Cannot set tile on given trace
    ErrorTileInvalid,
    // Cannot get back train
    ErrorTrainNotSet,
    // Cannot set train 'cause it's already set for current user
    ErrorTrainAlreadySet,
}

pub struct TrackData {
    tiles: Vec<Tile>,
    last_face: Face,
}

impl TrackData {
    pub fn new(last_face: Face) -> Self {
        Self {
            tiles: vec![],
            last_face,
        }
    }

    pub fn put_tile(&mut self, tile: Tile) -> bool {
        match tile.can_be_stacked(self.last_face) {
            None => false,
            Some(new_last_face) => {
                self.last_face = new_last_face;
                self.tiles.push(tile);

                true
            }
        }
    }
}

pub struct GameState {
    players: Vec<ActorId>,
    tracks: Vec<(TrackData, bool)>,
    start_tile: u32,
    current_player: u32,
}

impl GameState {
    pub fn skip_turn(&mut self, player: ActorId) {
        let i = self.current_player as usize;
        if self.players[i] != player {
            unreachable!("it is not your turn");
        }

        self.current_player = i + 1;
        tracks[i].1 = true;

        self.post_actions();
    }

    fn post_actions(&mut self) {
        todo!()
    }

    pub fn make_turn(&mut self, player: ActorId, tile_id: u32, track_id: u32) {
        let i = self.current_player as usize;
        if self.players[i] != player {
            unreachable!("it is not your turn");
        }

        // check player owns the tile

        // check tile can be put on the track

        self.post_actions();
    }
}
