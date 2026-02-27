use crate::chess::{CastlingRights, ChessPiece, Color, initial_board};
use crate::xiangqi::{XiangqiColor, XiangqiPiece};

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Player {
    X,
    O,
}

#[derive(Clone, PartialEq)]
pub enum GameType {
    None,
    TicTacToe,
    Gomoku,
    Chess,
    Xiangqi,
}

#[derive(Clone, PartialEq)]
pub enum GameMode {
    PvE,
    PvP,
}

pub struct AppState {
    pub game_type: GameType,
    pub game_mode: GameMode,
    pub current_turn: Player,
    pub ttt_board: [Option<Player>; 9],
    pub gomoku_board: [Option<Player>; 225],
    pub game_over: bool,
    pub ai_thinking: bool,
    pub last_move: Option<usize>,
    // Chess fields
    pub chess_board: [Option<ChessPiece>; 64],
    pub selected_cell: Option<usize>,
    pub valid_moves: Vec<usize>,
    pub chess_turn: Color,
    pub castling_rights: CastlingRights,
    pub en_passant_target: Option<usize>,
    pub last_move_from: Option<usize>,
    pub last_move_to: Option<usize>,
    // Xiangqi fields
    pub xiangqi_board: [Option<XiangqiPiece>; 90],
    pub xiangqi_selected: Option<usize>,
    pub xiangqi_valid_moves: Vec<usize>,
    pub xiangqi_turn: XiangqiColor,
    pub xiangqi_last_from: Option<usize>,
    pub xiangqi_last_to: Option<usize>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            game_type: GameType::None,
            game_mode: GameMode::PvE,
            current_turn: Player::X,
            ttt_board: [None; 9],
            gomoku_board: [None; 225],
            game_over: false,
            ai_thinking: false,
            last_move: None,
            chess_board: initial_board(),
            selected_cell: None,
            valid_moves: Vec::new(),
            chess_turn: Color::White,
            castling_rights: CastlingRights::default(),
            en_passant_target: None,
            last_move_from: None,
            last_move_to: None,
            xiangqi_board: crate::xiangqi::initial_board(),
            xiangqi_selected: None,
            xiangqi_valid_moves: Vec::new(),
            xiangqi_turn: XiangqiColor::Red,
            xiangqi_last_from: None,
            xiangqi_last_to: None,
        }
    }
}
