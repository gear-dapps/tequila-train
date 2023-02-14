#![no_std]

use gmeta::{In, Metadata};
use gstd::{prelude::*, ActorId};

pub struct ContractMetadata;

impl Metadata for ContractMetadata {
    type Init = In<Vec<ActorId>>;
    type Handle = In<Command>;
    type Others = ();
    type Reply = ();
    type Signal = ();
    type State = GameState;
}

#[derive(Encode, Decode, TypeInfo, Hash, PartialEq, PartialOrd, Eq, Ord, Clone, Debug, Default)]
pub struct State(pub Vec<(ActorId, u128)>);

#[derive(
    Debug,
    Clone,
    Copy,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    enum_iterator::Sequence,
    TypeInfo,
    Encode,
    Decode,
)]
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, TypeInfo, Encode, Decode)]
pub struct Tile {
    pub left: Face,
    pub right: Face,
}

impl Tile {
    pub fn new(left: Face, right: Face) -> Self {
        Self { left, right }
    }

    pub fn swap(self) -> Self {
        Self {
            left: self.right,
            right: self.left,
        }
    }

    pub fn is_double(&self) -> bool {
        self.left == self.right
    }

    pub fn can_adjoin(&self, other: &Tile) -> bool {
        self.right == other.left
    }
}

pub fn build_tile_collection() -> Vec<Tile> {
    enum_iterator::all::<Face>()
        .enumerate()
        .flat_map(|(i, face_first)| {
            enum_iterator::all::<Face>()
                .skip(i)
                .map(move |face_second| Tile::new(face_first, face_second))
        })
        .collect()
}

#[derive(Encode, Decode, TypeInfo, Hash, PartialEq, PartialOrd, Eq, Ord, Clone, Debug)]
pub struct Players {
    players: Vec<ActorId>,
}

#[derive(Encode, Decode, TypeInfo, Hash, PartialEq, PartialOrd, Eq, Ord, Clone, Copy, Debug)]
pub enum Command {
    Skip,
    Place {
        tile_id: u32,
        track_id: u32,
        remove_train: bool,
    },
}

#[derive(Debug, TypeInfo, Encode, Decode, Clone, Default)]
pub struct TrackData {
    pub tiles: Vec<Tile>,
    pub has_train: bool,
}

#[derive(Debug, TypeInfo, Encode, Decode, Clone)]
pub struct GameState {
    pub players: Vec<ActorId>,
    pub tracks: Vec<TrackData>,
    pub shots: Vec<u32>,
    pub start_tile: u32,
    pub current_player: u32,
    tile_to_player: BTreeMap<u32, u32>,
    pub tiles: Vec<Tile>,
    _remaining_tiles: BTreeSet<u32>,
    winner: Option<ActorId>,
}

/// - 2..4 players: 8 tiles
/// - 5 players: 7 tiles
/// - 6 players: 6 tiles
/// - 7 players: 5 tiles
/// - 8 players: 4 tiles
fn tiles_per_person(players_amount: usize) -> usize {
    match players_amount {
        2..=4 => 8,
        5..=8 => 12 - players_amount,
        _ => unreachable!("Invalid player amount reached"),
    }
}

fn get_random_from_set<T: Copy>(set: &BTreeSet<T>) -> T {
    let index = 0; // TODO: Make it random
    *set.iter().nth(index).unwrap()
}

impl GameState {
    // TODO: cover it with tests
    pub fn new(initial_data: &Players) -> Option<GameState> {
        // Check that players amount is allowed
        let players_amount = initial_data.players.len();
        if !(2..=8).contains(&players_amount) {
            return None;
        }

        let start_tile: u32 = 0; // TODO: identify
        let current_player: u32 = 0; // TODO: identify
        let mut tile_to_player: BTreeMap<u32, u32> = Default::default();
        let shots = vec![0u32; players_amount];

        // Build all possible tiles
        let tiles = build_tile_collection();
        let mut remaining_tiles: BTreeSet<u32> = Default::default();
        for index in 0..tiles.len() {
            remaining_tiles.insert(index as u32);
        }

        // Spread tiles to players
        let tiles_per_person = tiles_per_person(players_amount);
        for (player_index, _) in initial_data.players.iter().enumerate() {
            for _ in 1..=tiles_per_person {
                let tile_id = get_random_from_set(&remaining_tiles);
                remaining_tiles.remove(&tile_id);

                tile_to_player.insert(tile_id, player_index as u32);
            }
        }

        Some(GameState {
            players: initial_data.players.clone(),
            tracks: vec![Default::default(); players_amount],
            shots,
            start_tile,
            current_player,
            tile_to_player,
            tiles,
            _remaining_tiles: remaining_tiles,
            winner: None,
        })
    }

    pub fn winner(&self) -> Option<ActorId> {
        self.winner
    }

    pub fn skip_turn(&mut self, player: ActorId) {
        let i = self.current_player as usize;
        if self.players[i] != player {
            unreachable!("it is not your turn");
        }

        self.tracks[i].has_train = true;

        self.post_actions();
    }

    fn post_actions(&mut self) {
        let i = self.current_player as usize;
        self.current_player = match i + 1 >= self.players.len() {
            true => 0,
            false => (i + 1) as u32,
        };

        todo!()
    }

    pub fn make_turn(&mut self, player: ActorId, tile_id: u32, track_id: u32, remove_train: bool) {
        let i = self.current_player as usize;
        if self.players[i] != player {
            unreachable!("it is not your turn");
        }

        // check player owns the tile
        match self.tile_to_player.get(&tile_id) {
            None => unreachable!("invalid tile id"),
            Some(user_id) if *user_id != self.current_player => unreachable!("wrong tile owner"),
            _ => (),
        }

        // check tile can be put on the track
        if track_id != self.current_player
            && self
                .tracks
                .get(track_id as usize)
                .map_or(false, |data| data.has_train)
        {
            unreachable!("invalid track");
        }

        let tile = self.tiles[tile_id as usize];
        if !self.put_tile(tile, track_id as usize) {
            unreachable!("invalid tile");
        }

        // remove train if all criterea met
        if remove_train && track_id == self.current_player {
            self.tracks[i].has_train = false;
            self.shots[i] += 1;
        }

        // remove tile from player's set
        self.tile_to_player.remove(&tile_id);

        self.post_actions();
    }

    fn put_tile(&mut self, tile: Tile, track_id: usize) -> bool {
        let track = &mut self.tracks[track_id];
        let last_tile = match track.tiles.last() {
            None => &self.tiles[self.start_tile as usize],
            Some(tile) => tile,
        };

        if last_tile.can_adjoin(&tile) {
            track.tiles.push(tile);
            return true;
        }

        let tile = tile.swap();
        if last_tile.can_adjoin(&tile) {
            track.tiles.push(tile);
            return true;
        }

        false
    }
}
