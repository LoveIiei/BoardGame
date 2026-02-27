use super::moves::{ChessMove, apply_move, generate_legal_moves, is_checkmate, is_stalemate};
use super::{CastlingRights, ChessPiece, Color, PieceType};

const PAWN_VALUE: i32 = 100;
const KNIGHT_VALUE: i32 = 320;
const BISHOP_VALUE: i32 = 330;
const ROOK_VALUE: i32 = 500;
const QUEEN_VALUE: i32 = 900;
const KING_VALUE: i32 = 20000;

fn piece_value(pt: PieceType) -> i32 {
    match pt {
        PieceType::Pawn => PAWN_VALUE,
        PieceType::Knight => KNIGHT_VALUE,
        PieceType::Bishop => BISHOP_VALUE,
        PieceType::Rook => ROOK_VALUE,
        PieceType::Queen => QUEEN_VALUE,
        PieceType::King => KING_VALUE,
    }
}

// Piece-square tables (from White's perspective, index 0 = rank 0 = top = Black's back rank)
// For Black pieces, we mirror vertically (index = 63 - sq).

#[rustfmt::skip]
const PAWN_TABLE: [i32; 64] = [
     0,  0,  0,  0,  0,  0,  0,  0,
    50, 50, 50, 50, 50, 50, 50, 50,
    10, 10, 20, 30, 30, 20, 10, 10,
     5,  5, 10, 25, 25, 10,  5,  5,
     0,  0,  0, 20, 20,  0,  0,  0,
     5, -5,-10,  0,  0,-10, -5,  5,
     5, 10, 10,-20,-20, 10, 10,  5,
     0,  0,  0,  0,  0,  0,  0,  0,
];

#[rustfmt::skip]
const KNIGHT_TABLE: [i32; 64] = [
    -50,-40,-30,-30,-30,-30,-40,-50,
    -40,-20,  0,  0,  0,  0,-20,-40,
    -30,  0, 10, 15, 15, 10,  0,-30,
    -30,  5, 15, 20, 20, 15,  5,-30,
    -30,  0, 15, 20, 20, 15,  0,-30,
    -30,  5, 10, 15, 15, 10,  5,-30,
    -40,-20,  0,  5,  5,  0,-20,-40,
    -50,-40,-30,-30,-30,-30,-40,-50,
];

#[rustfmt::skip]
const BISHOP_TABLE: [i32; 64] = [
    -20,-10,-10,-10,-10,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0, 10, 10, 10, 10,  0,-10,
    -10,  5,  5, 10, 10,  5,  5,-10,
    -10,  0, 10, 10, 10, 10,  0,-10,
    -10, 10, 10, 10, 10, 10, 10,-10,
    -10,  5,  0,  0,  0,  0,  5,-10,
    -20,-10,-10,-10,-10,-10,-10,-20,
];

#[rustfmt::skip]
const ROOK_TABLE: [i32; 64] = [
     0,  0,  0,  0,  0,  0,  0,  0,
     5, 10, 10, 10, 10, 10, 10,  5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
     0,  0,  0,  5,  5,  0,  0,  0,
];

#[rustfmt::skip]
const QUEEN_TABLE: [i32; 64] = [
    -20,-10,-10, -5, -5,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5,  5,  5,  5,  0,-10,
     -5,  0,  5,  5,  5,  5,  0, -5,
      0,  0,  5,  5,  5,  5,  0, -5,
    -10,  5,  5,  5,  5,  5,  0,-10,
    -10,  0,  5,  0,  0,  0,  0,-10,
    -20,-10,-10, -5, -5,-10,-10,-20,
];

#[rustfmt::skip]
const KING_TABLE: [i32; 64] = [
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -20,-30,-30,-40,-40,-30,-30,-20,
    -10,-20,-20,-20,-20,-20,-20,-10,
     20, 20,  0,  0,  0,  0, 20, 20,
     20, 30, 10,  0,  0, 10, 30, 20,
];

fn pst_value(piece_type: PieceType, sq: usize) -> i32 {
    match piece_type {
        PieceType::Pawn => PAWN_TABLE[sq],
        PieceType::Knight => KNIGHT_TABLE[sq],
        PieceType::Bishop => BISHOP_TABLE[sq],
        PieceType::Rook => ROOK_TABLE[sq],
        PieceType::Queen => QUEEN_TABLE[sq],
        PieceType::King => KING_TABLE[sq],
    }
}

/// Evaluate board from White's perspective. Positive = White winning.
pub fn evaluate(board: &[Option<ChessPiece>; 64]) -> i32 {
    let mut score = 0;
    for (sq, cell) in board.iter().enumerate() {
        if let Some(piece) = cell {
            let material = piece_value(piece.piece_type);
            // For White, use the table index directly.
            // For Black, mirror vertically: 63 - sq flips rank while keeping file.
            // Actually, to mirror rank only: mirrored = (7 - rank) * 8 + file
            let pst = if piece.color == Color::White {
                pst_value(piece.piece_type, sq)
            } else {
                let rank = sq / 8;
                let file = sq % 8;
                let mirrored = (7 - rank) * 8 + file;
                pst_value(piece.piece_type, mirrored)
            };

            match piece.color {
                Color::White => score += material + pst,
                Color::Black => score -= material + pst,
            }
        }
    }
    score
}

/// Order moves for better alpha-beta pruning: captures first (MVV-LVA).
fn order_moves(board: &[Option<ChessPiece>; 64], moves: &mut [ChessMove]) {
    moves.sort_by(|a, b| {
        let score_a = move_order_score(board, a);
        let score_b = move_order_score(board, b);
        score_b.cmp(&score_a) // Higher score first
    });
}

fn move_order_score(board: &[Option<ChessPiece>; 64], mv: &ChessMove) -> i32 {
    let mut score = 0;
    // Captures: MVV-LVA (Most Valuable Victim - Least Valuable Attacker)
    if let Some(captured) = board[mv.to] {
        let attacker = board[mv.from].unwrap();
        score += 10 * piece_value(captured.piece_type) - piece_value(attacker.piece_type);
    }
    // Promotions are very valuable
    if mv.promotion.is_some() {
        score += QUEEN_VALUE;
    }
    score
}

#[allow(clippy::too_many_arguments)]
fn alpha_beta(
    board: &[Option<ChessPiece>; 64],
    color: Color,
    cr: &CastlingRights,
    ep: Option<usize>,
    depth: i32,
    mut alpha: i32,
    mut beta: i32,
    maximizing: bool,
) -> i32 {
    if depth == 0 {
        return evaluate(board);
    }

    // Terminal node checks
    if is_checkmate(board, color, cr, ep) {
        // The side to move is in checkmate — they lose
        return if maximizing {
            -100000 - depth // Losing sooner is worse
        } else {
            100000 + depth // Opponent losing sooner is better for maximizer
        };
    }
    if is_stalemate(board, color, cr, ep) {
        return 0;
    }

    let mut moves = generate_legal_moves(board, color, cr, ep);
    order_moves(board, &mut moves);

    if maximizing {
        let mut max_eval = i32::MIN + 1;
        for mv in &moves {
            let (new_board, new_cr, new_ep) = apply_move(board, mv, cr, ep);
            let eval = alpha_beta(
                &new_board,
                color.opposite(),
                &new_cr,
                new_ep,
                depth - 1,
                alpha,
                beta,
                false,
            );
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
            let (new_board, new_cr, new_ep) = apply_move(board, mv, cr, ep);
            let eval = alpha_beta(
                &new_board,
                color.opposite(),
                &new_cr,
                new_ep,
                depth - 1,
                alpha,
                beta,
                true,
            );
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
pub fn get_best_chess_move(
    board: &[Option<ChessPiece>; 64],
    turn: Color,
    cr: &CastlingRights,
    ep: Option<usize>,
) -> Option<ChessMove> {
    let mut moves = generate_legal_moves(board, turn, cr, ep);
    if moves.is_empty() {
        return None;
    }

    order_moves(board, &mut moves);

    let maximizing = turn == Color::White;
    let mut best_score = if maximizing { i32::MIN + 1 } else { i32::MAX - 1 };
    let mut best_move = None;

    for mv in &moves {
        let (new_board, new_cr, new_ep) = apply_move(board, mv, cr, ep);
        let score = alpha_beta(
            &new_board,
            turn.opposite(),
            &new_cr,
            new_ep,
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
