use super::moves::{apply_move, generate_legal_moves, XiangqiMove};
use super::{PieceType, XiangqiColor, XiangqiPiece};

const SOLDIER_VALUE: i32 = 100;
const ADVISOR_VALUE: i32 = 200;
const ELEPHANT_VALUE: i32 = 200;
const HORSE_VALUE: i32 = 400;
const CANNON_VALUE: i32 = 450;
const CHARIOT_VALUE: i32 = 900;
const GENERAL_VALUE: i32 = 20000;

fn piece_value(pt: PieceType) -> i32 {
    match pt {
        PieceType::Soldier => SOLDIER_VALUE,
        PieceType::Advisor => ADVISOR_VALUE,
        PieceType::Elephant => ELEPHANT_VALUE,
        PieceType::Horse => HORSE_VALUE,
        PieceType::Cannon => CANNON_VALUE,
        PieceType::Chariot => CHARIOT_VALUE,
        PieceType::General => GENERAL_VALUE,
    }
}

// Piece-square tables from Red's perspective (90 entries).
// Row 0 = top = Black side, Row 9 = bottom = Red side.
// For Black pieces, mirror vertically: mirrored = (9 - row) * 9 + col.

#[rustfmt::skip]
const SOLDIER_TABLE: [i32; 90] = [
     0,  0,  0,  0,  0,  0,  0,  0,  0,
     0,  0,  0,  0,  0,  0,  0,  0,  0,
     0,  0,  0,  0,  0,  0,  0,  0,  0,
    10, 10, 20, 30, 40, 30, 20, 10, 10,
    20, 20, 30, 40, 50, 40, 30, 20, 20,
    10, 10, 20, 30, 30, 30, 20, 10, 10,
     0,  0,  0,  0,  0,  0,  0,  0,  0,
     0,  0,  0,  0,  0,  0,  0,  0,  0,
     0,  0,  0,  0,  0,  0,  0,  0,  0,
     0,  0,  0,  0,  0,  0,  0,  0,  0,
];

#[rustfmt::skip]
const HORSE_TABLE: [i32; 90] = [
   -10,  0,  0,  0,  0,  0,  0,  0,-10,
     0,  0, 10,  0,  0,  0, 10,  0,  0,
     0, 10, 20, 20, 10, 20, 20, 10,  0,
     0, 10, 20, 30, 30, 30, 20, 10,  0,
     0, 10, 20, 30, 30, 30, 20, 10,  0,
     0, 10, 20, 30, 30, 30, 20, 10,  0,
     0, 10, 20, 20, 10, 20, 20, 10,  0,
     0,  0, 10,  0,  0,  0, 10,  0,  0,
   -10,  0,  0,  0,  0,  0,  0,  0,-10,
   -20,-10,  0,  0,  0,  0,  0,-10,-20,
];

#[rustfmt::skip]
const CANNON_TABLE: [i32; 90] = [
     0, 10,  0, 10,  0, 10,  0, 10,  0,
     0,  0,  0,  0, 20,  0,  0,  0,  0,
     0,  0,  0,  0,  0,  0,  0,  0,  0,
     0,  0,  0,  0,  0,  0,  0,  0,  0,
     0,  0,  0,  0,  0,  0,  0,  0,  0,
     0,  0,  0,  0,  0,  0,  0,  0,  0,
     0,  0,  0,  0,  0,  0,  0,  0,  0,
     0,  0,  0,  0, 20,  0,  0,  0,  0,
     0, 10,  0, 10,  0, 10,  0, 10,  0,
    10, 10,  0, 20,  0, 20,  0, 10, 10,
];

#[rustfmt::skip]
const CHARIOT_TABLE: [i32; 90] = [
    10, 10, 10, 20, 20, 20, 10, 10, 10,
    20, 20, 30, 30, 30, 30, 30, 20, 20,
    10, 10, 20, 20, 20, 20, 20, 10, 10,
     0,  0, 10, 10, 10, 10, 10,  0,  0,
     0,  0, 10, 10, 10, 10, 10,  0,  0,
     0,  0, 10, 10, 10, 10, 10,  0,  0,
     0,  0, 10, 10, 10, 10, 10,  0,  0,
    10, 10, 20, 20, 20, 20, 20, 10, 10,
    10, 10, 10, 10, 10, 10, 10, 10, 10,
     0,  0,  0, 10, 10, 10,  0,  0,  0,
];

#[rustfmt::skip]
const GENERAL_TABLE: [i32; 90] = [
     0,  0,  0,  0,  0,  0,  0,  0,  0,
     0,  0,  0,  0,  0,  0,  0,  0,  0,
     0,  0,  0,  0,  0,  0,  0,  0,  0,
     0,  0,  0,  0,  0,  0,  0,  0,  0,
     0,  0,  0,  0,  0,  0,  0,  0,  0,
     0,  0,  0,  0,  0,  0,  0,  0,  0,
     0,  0,  0,  0,  0,  0,  0,  0,  0,
     0,  0,  0,  5,  5,  5,  0,  0,  0,
     0,  0,  0, 10, 10, 10,  0,  0,  0,
     0,  0,  0, 10, 15, 10,  0,  0,  0,
];

#[rustfmt::skip]
const ADVISOR_TABLE: [i32; 90] = [
     0,  0,  0,  0,  0,  0,  0,  0,  0,
     0,  0,  0,  0,  0,  0,  0,  0,  0,
     0,  0,  0,  0,  0,  0,  0,  0,  0,
     0,  0,  0,  0,  0,  0,  0,  0,  0,
     0,  0,  0,  0,  0,  0,  0,  0,  0,
     0,  0,  0,  0,  0,  0,  0,  0,  0,
     0,  0,  0,  0,  0,  0,  0,  0,  0,
     0,  0,  0, 10,  0, 10,  0,  0,  0,
     0,  0,  0,  0, 15,  0,  0,  0,  0,
     0,  0,  0, 10,  0, 10,  0,  0,  0,
];

#[rustfmt::skip]
const ELEPHANT_TABLE: [i32; 90] = [
     0,  0,  0,  0,  0,  0,  0,  0,  0,
     0,  0,  0,  0,  0,  0,  0,  0,  0,
     0,  0,  0,  0,  0,  0,  0,  0,  0,
     0,  0,  0,  0,  0,  0,  0,  0,  0,
     0,  0,  0,  0,  0,  0,  0,  0,  0,
     0, 10,  0,  0,  0,  0,  0, 10,  0,
     0,  0,  0,  0, 10,  0,  0,  0,  0,
     0,  0, 15,  0,  0,  0, 15,  0,  0,
     0,  0,  0,  0,  0,  0,  0,  0,  0,
     0,  0, 10,  0,  0,  0, 10,  0,  0,
];

fn pst_value(piece_type: PieceType, sq: usize) -> i32 {
    match piece_type {
        PieceType::Soldier => SOLDIER_TABLE[sq],
        PieceType::Horse => HORSE_TABLE[sq],
        PieceType::Cannon => CANNON_TABLE[sq],
        PieceType::Chariot => CHARIOT_TABLE[sq],
        PieceType::General => GENERAL_TABLE[sq],
        PieceType::Advisor => ADVISOR_TABLE[sq],
        PieceType::Elephant => ELEPHANT_TABLE[sq],
    }
}

/// Evaluate board from Red's perspective. Positive = Red winning.
pub fn evaluate(board: &[Option<XiangqiPiece>; 90]) -> i32 {
    let mut score = 0;
    for (sq, cell) in board.iter().enumerate() {
        if let Some(piece) = cell {
            let material = piece_value(piece.piece_type);
            let pst = if piece.color == XiangqiColor::Red {
                pst_value(piece.piece_type, sq)
            } else {
                let row = sq / 9;
                let col = sq % 9;
                let mirrored = (9 - row) * 9 + col;
                pst_value(piece.piece_type, mirrored)
            };
            match piece.color {
                XiangqiColor::Red => score += material + pst,
                XiangqiColor::Black => score -= material + pst,
            }
        }
    }
    score
}

fn order_moves(board: &[Option<XiangqiPiece>; 90], moves: &mut [XiangqiMove]) {
    moves.sort_by(|a, b| {
        let score_a = move_order_score(board, a);
        let score_b = move_order_score(board, b);
        score_b.cmp(&score_a)
    });
}

fn move_order_score(board: &[Option<XiangqiPiece>; 90], mv: &XiangqiMove) -> i32 {
    let mut score = 0;
    if let Some(captured) = board[mv.to] {
        let attacker = board[mv.from].unwrap();
        score += 10 * piece_value(captured.piece_type) - piece_value(attacker.piece_type);
    }
    score
}

fn alpha_beta(
    board: &[Option<XiangqiPiece>; 90],
    color: XiangqiColor,
    depth: i32,
    mut alpha: i32,
    mut beta: i32,
    maximizing: bool,
) -> i32 {
    if depth == 0 {
        return evaluate(board);
    }

    let mut moves = generate_legal_moves(board, color);

    if moves.is_empty() {
        // No legal moves: checkmate or stalemate (both are losses in xiangqi)
        return if maximizing {
            -100000 - depth
        } else {
            100000 + depth
        };
    }

    order_moves(board, &mut moves);

    if maximizing {
        let mut max_eval = i32::MIN + 1;
        for mv in &moves {
            let new_board = apply_move(board, mv);
            let eval =
                alpha_beta(&new_board, color.opposite(), depth - 1, alpha, beta, false);
            max_eval = max_eval.max(eval);
            alpha = alpha.max(eval);
            if beta <= alpha {
                break;
            }
        }
        max_eval
    } else {
        let mut min_eval = i32::MAX - 1;
        for mv in &moves {
            let new_board = apply_move(board, mv);
            let eval =
                alpha_beta(&new_board, color.opposite(), depth - 1, alpha, beta, true);
            min_eval = min_eval.min(eval);
            beta = beta.min(eval);
            if beta <= alpha {
                break;
            }
        }
        min_eval
    }
}

/// Find the best move for the given side using alpha-beta search at depth 3.
pub fn get_best_xiangqi_move(
    board: &[Option<XiangqiPiece>; 90],
    turn: XiangqiColor,
) -> Option<XiangqiMove> {
    let mut moves = generate_legal_moves(board, turn);
    if moves.is_empty() {
        return None;
    }

    order_moves(board, &mut moves);

    let maximizing = turn == XiangqiColor::Red;
    let mut best_score = if maximizing {
        i32::MIN + 1
    } else {
        i32::MAX - 1
    };
    let mut best_move = None;

    for mv in &moves {
        let new_board = apply_move(board, mv);
        let score = alpha_beta(
            &new_board,
            turn.opposite(),
            2, // depth-1 since we already made one move
            i32::MIN + 1,
            i32::MAX - 1,
            !maximizing,
        );

        let is_better = if maximizing {
            score > best_score
        } else {
            score < best_score
        };

        if is_better {
            best_score = score;
            best_move = Some(*mv);
        }
    }

    best_move
}
