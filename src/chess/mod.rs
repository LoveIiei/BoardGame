pub mod ai;
pub mod moves;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PieceType {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn opposite(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct ChessPiece {
    pub piece_type: PieceType,
    pub color: Color,
}

impl ChessPiece {
    pub fn new(piece_type: PieceType, color: Color) -> Self {
        Self { piece_type, color }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct CastlingRights {
    pub white_king_side: bool,
    pub white_queen_side: bool,
    pub black_king_side: bool,
    pub black_queen_side: bool,
}

impl Default for CastlingRights {
    fn default() -> Self {
        Self {
            white_king_side: true,
            white_queen_side: true,
            black_king_side: true,
            black_queen_side: true,
        }
    }
}

/// Standard starting position.
/// Index = rank * 8 + file. Rank 0 = top (Black back rank), rank 7 = bottom (White back rank).
pub fn initial_board() -> [Option<ChessPiece>; 64] {
    let mut board = [None; 64];

    // Black back rank (rank 0, indices 0..8)
    let back_rank = [
        PieceType::Rook,
        PieceType::Knight,
        PieceType::Bishop,
        PieceType::Queen,
        PieceType::King,
        PieceType::Bishop,
        PieceType::Knight,
        PieceType::Rook,
    ];
    for (file, &pt) in back_rank.iter().enumerate() {
        board[file] = Some(ChessPiece::new(pt, Color::Black));
    }
    // Black pawns (rank 1, indices 8..16)
    for file in 0..8 {
        board[8 + file] = Some(ChessPiece::new(PieceType::Pawn, Color::Black));
    }

    // White pawns (rank 6, indices 48..56)
    for file in 0..8 {
        board[48 + file] = Some(ChessPiece::new(PieceType::Pawn, Color::White));
    }
    // White back rank (rank 7, indices 56..64)
    for (file, &pt) in back_rank.iter().enumerate() {
        board[56 + file] = Some(ChessPiece::new(pt, Color::White));
    }

    board
}

pub fn piece_to_unicode(piece: &ChessPiece) -> &'static str {
    match (piece.color, piece.piece_type) {
        (Color::White, PieceType::King) => "♔",
        (Color::White, PieceType::Queen) => "♕",
        (Color::White, PieceType::Rook) => "♖",
        (Color::White, PieceType::Bishop) => "♗",
        (Color::White, PieceType::Knight) => "♘",
        (Color::White, PieceType::Pawn) => "♙",
        (Color::Black, PieceType::King) => "♚",
        (Color::Black, PieceType::Queen) => "♛",
        (Color::Black, PieceType::Rook) => "♜",
        (Color::Black, PieceType::Bishop) => "♝",
        (Color::Black, PieceType::Knight) => "♞",
        (Color::Black, PieceType::Pawn) => "♟",
    }
}
