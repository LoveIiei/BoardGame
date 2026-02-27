pub mod ai;
pub mod moves;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PieceType {
    General,
    Advisor,
    Elephant,
    Horse,
    Chariot,
    Cannon,
    Soldier,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum XiangqiColor {
    Red,
    Black,
}

impl XiangqiColor {
    pub fn opposite(self) -> XiangqiColor {
        match self {
            XiangqiColor::Red => XiangqiColor::Black,
            XiangqiColor::Black => XiangqiColor::Red,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct XiangqiPiece {
    pub piece_type: PieceType,
    pub color: XiangqiColor,
}

impl XiangqiPiece {
    pub fn new(piece_type: PieceType, color: XiangqiColor) -> Self {
        Self { piece_type, color }
    }
}

/// Standard starting position.
/// Index = row * 9 + col. Row 0 = top (Black back rank), Row 9 = bottom (Red back rank).
/// River between rows 4 and 5.
pub fn initial_board() -> [Option<XiangqiPiece>; 90] {
    let mut board = [None; 90];

    // Black back rank (row 0)
    let back_rank = [
        PieceType::Chariot,
        PieceType::Horse,
        PieceType::Elephant,
        PieceType::Advisor,
        PieceType::General,
        PieceType::Advisor,
        PieceType::Elephant,
        PieceType::Horse,
        PieceType::Chariot,
    ];
    for (col, &pt) in back_rank.iter().enumerate() {
        board[col] = Some(XiangqiPiece::new(pt, XiangqiColor::Black));
    }

    // Black cannons (row 2, cols 1 and 7)
    board[2 * 9 + 1] = Some(XiangqiPiece::new(PieceType::Cannon, XiangqiColor::Black));
    board[2 * 9 + 7] = Some(XiangqiPiece::new(PieceType::Cannon, XiangqiColor::Black));

    // Black soldiers (row 3, cols 0,2,4,6,8)
    for col in (0..9).step_by(2) {
        board[3 * 9 + col] = Some(XiangqiPiece::new(PieceType::Soldier, XiangqiColor::Black));
    }

    // Red soldiers (row 6, cols 0,2,4,6,8)
    for col in (0..9).step_by(2) {
        board[6 * 9 + col] = Some(XiangqiPiece::new(PieceType::Soldier, XiangqiColor::Red));
    }

    // Red cannons (row 7, cols 1 and 7)
    board[7 * 9 + 1] = Some(XiangqiPiece::new(PieceType::Cannon, XiangqiColor::Red));
    board[7 * 9 + 7] = Some(XiangqiPiece::new(PieceType::Cannon, XiangqiColor::Red));

    // Red back rank (row 9)
    for (col, &pt) in back_rank.iter().enumerate() {
        board[9 * 9 + col] = Some(XiangqiPiece::new(pt, XiangqiColor::Red));
    }

    board
}

pub fn piece_to_text(piece: &XiangqiPiece) -> &'static str {
    match (piece.color, piece.piece_type) {
        (XiangqiColor::Red, PieceType::General) => "帥",
        (XiangqiColor::Red, PieceType::Advisor) => "仕",
        (XiangqiColor::Red, PieceType::Elephant) => "相",
        (XiangqiColor::Red, PieceType::Horse) => "傌",
        (XiangqiColor::Red, PieceType::Chariot) => "俥",
        (XiangqiColor::Red, PieceType::Cannon) => "炮",
        (XiangqiColor::Red, PieceType::Soldier) => "兵",
        (XiangqiColor::Black, PieceType::General) => "將",
        (XiangqiColor::Black, PieceType::Advisor) => "士",
        (XiangqiColor::Black, PieceType::Elephant) => "象",
        (XiangqiColor::Black, PieceType::Horse) => "馬",
        (XiangqiColor::Black, PieceType::Chariot) => "車",
        (XiangqiColor::Black, PieceType::Cannon) => "砲",
        (XiangqiColor::Black, PieceType::Soldier) => "卒",
    }
}
