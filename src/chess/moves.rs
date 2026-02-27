use super::{CastlingRights, ChessPiece, Color, PieceType};

#[derive(Copy, Clone, Debug)]
pub struct ChessMove {
    pub from: usize,
    pub to: usize,
    pub promotion: Option<PieceType>,
    pub is_castling: bool,
    pub is_en_passant: bool,
}

impl ChessMove {
    fn normal(from: usize, to: usize) -> Self {
        Self {
            from,
            to,
            promotion: None,
            is_castling: false,
            is_en_passant: false,
        }
    }
}

fn in_bounds(r: isize, f: isize) -> bool {
    (0..8).contains(&r) && (0..8).contains(&f)
}

fn idx(r: isize, f: isize) -> usize {
    r as usize * 8 + f as usize
}

pub fn find_king(board: &[Option<ChessPiece>; 64], color: Color) -> usize {
    board
        .iter()
        .position(|p| {
            matches!(p, Some(piece) if piece.color == color && piece.piece_type == PieceType::King)
        })
        .expect("King must exist on board")
}

pub fn is_square_attacked(board: &[Option<ChessPiece>; 64], square: usize, by_color: Color) -> bool {
    let rank = square / 8;
    let file = square % 8;

    // Knight attacks
    let knight_offsets: [(isize, isize); 8] = [
        (-2, -1), (-2, 1), (-1, -2), (-1, 2),
        (1, -2), (1, 2), (2, -1), (2, 1),
    ];
    for (dr, df) in knight_offsets {
        let nr = rank as isize + dr;
        let nf = file as isize + df;
        if in_bounds(nr, nf)
            && matches!(board[idx(nr, nf)], Some(p) if p.color == by_color && p.piece_type == PieceType::Knight)
        {
            return true;
        }
    }

    // Pawn attacks
    let pawn_dir: isize = if by_color == Color::White { 1 } else { -1 };
    let pawn_rank = rank as isize + pawn_dir;
    if (0..8).contains(&pawn_rank) {
        for df in [-1isize, 1] {
            let pf = file as isize + df;
            if (0..8).contains(&pf)
                && matches!(board[idx(pawn_rank, pf)], Some(p) if p.color == by_color && p.piece_type == PieceType::Pawn)
            {
                return true;
            }
        }
    }

    // King attacks
    for dr in -1isize..=1 {
        for df in -1isize..=1 {
            if dr == 0 && df == 0 {
                continue;
            }
            let nr = rank as isize + dr;
            let nf = file as isize + df;
            if in_bounds(nr, nf)
                && matches!(board[idx(nr, nf)], Some(p) if p.color == by_color && p.piece_type == PieceType::King)
            {
                return true;
            }
        }
    }

    // Sliding pieces: rook/queen along ranks and files
    let rook_dirs: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    for (dr, df) in rook_dirs {
        let mut nr = rank as isize + dr;
        let mut nf = file as isize + df;
        while in_bounds(nr, nf) {
            let i = idx(nr, nf);
            if let Some(piece) = board[i] {
                if piece.color == by_color
                    && (piece.piece_type == PieceType::Rook
                        || piece.piece_type == PieceType::Queen)
                {
                    return true;
                }
                break;
            }
            nr += dr;
            nf += df;
        }
    }

    // Sliding pieces: bishop/queen along diagonals
    let bishop_dirs: [(isize, isize); 4] = [(-1, -1), (-1, 1), (1, -1), (1, 1)];
    for (dr, df) in bishop_dirs {
        let mut nr = rank as isize + dr;
        let mut nf = file as isize + df;
        while in_bounds(nr, nf) {
            let i = idx(nr, nf);
            if let Some(piece) = board[i] {
                if piece.color == by_color
                    && (piece.piece_type == PieceType::Bishop
                        || piece.piece_type == PieceType::Queen)
                {
                    return true;
                }
                break;
            }
            nr += dr;
            nf += df;
        }
    }

    false
}

pub fn is_in_check(board: &[Option<ChessPiece>; 64], color: Color) -> bool {
    let king_sq = find_king(board, color);
    is_square_attacked(board, king_sq, color.opposite())
}

/// Generate pseudo-legal moves (before filtering for check).
fn generate_pseudo_legal_moves(
    board: &[Option<ChessPiece>; 64],
    color: Color,
    castling_rights: &CastlingRights,
    en_passant: Option<usize>,
) -> Vec<ChessMove> {
    let mut moves = Vec::new();

    for sq in 0..64 {
        let piece = match board[sq] {
            Some(p) if p.color == color => p,
            _ => continue,
        };

        let rank = sq / 8;
        let file = sq % 8;

        match piece.piece_type {
            PieceType::Pawn => {
                let dir: isize = if color == Color::White { -1 } else { 1 };
                let start_rank = if color == Color::White { 6 } else { 1 };
                let promo_rank = if color == Color::White { 0 } else { 7 };

                // Single push
                let fwd = (rank as isize + dir) as usize;
                if fwd < 8 {
                    let fwd_idx = fwd * 8 + file;
                    if board[fwd_idx].is_none() {
                        if fwd == promo_rank {
                            moves.push(ChessMove {
                                from: sq,
                                to: fwd_idx,
                                promotion: Some(PieceType::Queen),
                                is_castling: false,
                                is_en_passant: false,
                            });
                        } else {
                            moves.push(ChessMove::normal(sq, fwd_idx));
                        }

                        // Double push from starting rank
                        if rank == start_rank {
                            let dbl = (rank as isize + dir * 2) as usize;
                            let dbl_idx = dbl * 8 + file;
                            if board[dbl_idx].is_none() {
                                moves.push(ChessMove::normal(sq, dbl_idx));
                            }
                        }
                    }
                }

                // Captures (including en passant)
                for df in [-1isize, 1] {
                    let nf = file as isize + df;
                    if !(0..8).contains(&nf) {
                        continue;
                    }
                    let nr = (rank as isize + dir) as usize;
                    if nr >= 8 {
                        continue;
                    }
                    let cap_idx = nr * 8 + nf as usize;

                    let is_capture = board[cap_idx].is_some_and(|p| p.color != color);
                    let is_ep = en_passant == Some(cap_idx);

                    if is_capture || is_ep {
                        if nr == promo_rank {
                            moves.push(ChessMove {
                                from: sq,
                                to: cap_idx,
                                promotion: Some(PieceType::Queen),
                                is_castling: false,
                                is_en_passant: is_ep,
                            });
                        } else {
                            moves.push(ChessMove {
                                from: sq,
                                to: cap_idx,
                                promotion: None,
                                is_castling: false,
                                is_en_passant: is_ep,
                            });
                        }
                    }
                }
            }

            PieceType::Knight => {
                let offsets: [(isize, isize); 8] = [
                    (-2, -1), (-2, 1), (-1, -2), (-1, 2),
                    (1, -2), (1, 2), (2, -1), (2, 1),
                ];
                for (dr, df) in offsets {
                    let nr = rank as isize + dr;
                    let nf = file as isize + df;
                    if in_bounds(nr, nf) {
                        let i = idx(nr, nf);
                        if board[i].is_none_or(|p| p.color != color) {
                            moves.push(ChessMove::normal(sq, i));
                        }
                    }
                }
            }

            PieceType::Bishop => {
                let dirs: [(isize, isize); 4] = [(-1, -1), (-1, 1), (1, -1), (1, 1)];
                for (dr, df) in dirs {
                    generate_sliding_moves(board, sq, dr, df, color, &mut moves);
                }
            }

            PieceType::Rook => {
                let dirs: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
                for (dr, df) in dirs {
                    generate_sliding_moves(board, sq, dr, df, color, &mut moves);
                }
            }

            PieceType::Queen => {
                let dirs: [(isize, isize); 8] = [
                    (-1, -1), (-1, 0), (-1, 1), (0, -1),
                    (0, 1), (1, -1), (1, 0), (1, 1),
                ];
                for (dr, df) in dirs {
                    generate_sliding_moves(board, sq, dr, df, color, &mut moves);
                }
            }

            PieceType::King => {
                for dr in -1isize..=1 {
                    for df in -1isize..=1 {
                        if dr == 0 && df == 0 {
                            continue;
                        }
                        let nr = rank as isize + dr;
                        let nf = file as isize + df;
                        if in_bounds(nr, nf) {
                            let i = idx(nr, nf);
                            if board[i].is_none_or(|p| p.color != color) {
                                moves.push(ChessMove::normal(sq, i));
                            }
                        }
                    }
                }

                // Castling
                if color == Color::White && sq == 60 {
                    if castling_rights.white_king_side
                        && board[61].is_none()
                        && board[62].is_none()
                        && !is_square_attacked(board, 60, Color::Black)
                        && !is_square_attacked(board, 61, Color::Black)
                        && !is_square_attacked(board, 62, Color::Black)
                    {
                        moves.push(ChessMove {
                            from: 60,
                            to: 62,
                            promotion: None,
                            is_castling: true,
                            is_en_passant: false,
                        });
                    }
                    if castling_rights.white_queen_side
                        && board[59].is_none()
                        && board[58].is_none()
                        && board[57].is_none()
                        && !is_square_attacked(board, 60, Color::Black)
                        && !is_square_attacked(board, 59, Color::Black)
                        && !is_square_attacked(board, 58, Color::Black)
                    {
                        moves.push(ChessMove {
                            from: 60,
                            to: 58,
                            promotion: None,
                            is_castling: true,
                            is_en_passant: false,
                        });
                    }
                } else if color == Color::Black && sq == 4 {
                    if castling_rights.black_king_side
                        && board[5].is_none()
                        && board[6].is_none()
                        && !is_square_attacked(board, 4, Color::White)
                        && !is_square_attacked(board, 5, Color::White)
                        && !is_square_attacked(board, 6, Color::White)
                    {
                        moves.push(ChessMove {
                            from: 4,
                            to: 6,
                            promotion: None,
                            is_castling: true,
                            is_en_passant: false,
                        });
                    }
                    if castling_rights.black_queen_side
                        && board[3].is_none()
                        && board[2].is_none()
                        && board[1].is_none()
                        && !is_square_attacked(board, 4, Color::White)
                        && !is_square_attacked(board, 3, Color::White)
                        && !is_square_attacked(board, 2, Color::White)
                    {
                        moves.push(ChessMove {
                            from: 4,
                            to: 2,
                            promotion: None,
                            is_castling: true,
                            is_en_passant: false,
                        });
                    }
                }
            }
        }
    }

    moves
}

fn generate_sliding_moves(
    board: &[Option<ChessPiece>; 64],
    sq: usize,
    dr: isize,
    df: isize,
    color: Color,
    moves: &mut Vec<ChessMove>,
) {
    let rank = sq / 8;
    let file = sq % 8;
    let mut nr = rank as isize + dr;
    let mut nf = file as isize + df;
    while in_bounds(nr, nf) {
        let i = idx(nr, nf);
        match board[i] {
            None => moves.push(ChessMove::normal(sq, i)),
            Some(p) if p.color != color => {
                moves.push(ChessMove::normal(sq, i));
                break;
            }
            _ => break,
        }
        nr += dr;
        nf += df;
    }
}

/// Apply a move and return (new_board, new_castling_rights, new_en_passant_target).
pub fn apply_move(
    board: &[Option<ChessPiece>; 64],
    mv: &ChessMove,
    castling_rights: &CastlingRights,
    _en_passant: Option<usize>,
) -> ([Option<ChessPiece>; 64], CastlingRights, Option<usize>) {
    let mut new_board = *board;
    let mut new_cr = *castling_rights;
    let mut new_ep = None;

    let piece = board[mv.from].expect("Moving from empty square");

    if mv.is_en_passant {
        let captured_rank = mv.from / 8;
        let captured_file = mv.to % 8;
        new_board[captured_rank * 8 + captured_file] = None;
    }

    if mv.is_castling {
        match mv.to {
            62 => {
                new_board[63] = None;
                new_board[61] = Some(ChessPiece::new(PieceType::Rook, Color::White));
            }
            58 => {
                new_board[56] = None;
                new_board[59] = Some(ChessPiece::new(PieceType::Rook, Color::White));
            }
            6 => {
                new_board[7] = None;
                new_board[5] = Some(ChessPiece::new(PieceType::Rook, Color::Black));
            }
            2 => {
                new_board[0] = None;
                new_board[3] = Some(ChessPiece::new(PieceType::Rook, Color::Black));
            }
            _ => {}
        }
    }

    new_board[mv.from] = None;
    if let Some(promo) = mv.promotion {
        new_board[mv.to] = Some(ChessPiece::new(promo, piece.color));
    } else {
        new_board[mv.to] = Some(piece);
    }

    if piece.piece_type == PieceType::Pawn {
        let from_rank = mv.from / 8;
        let to_rank = mv.to / 8;
        if from_rank.abs_diff(to_rank) == 2 {
            new_ep = Some((mv.from + mv.to) / 2);
        }
    }

    if piece.piece_type == PieceType::King {
        match piece.color {
            Color::White => {
                new_cr.white_king_side = false;
                new_cr.white_queen_side = false;
            }
            Color::Black => {
                new_cr.black_king_side = false;
                new_cr.black_queen_side = false;
            }
        }
    }
    match mv.from {
        56 => new_cr.white_queen_side = false,
        63 => new_cr.white_king_side = false,
        0 => new_cr.black_queen_side = false,
        7 => new_cr.black_king_side = false,
        _ => {}
    }
    match mv.to {
        56 => new_cr.white_queen_side = false,
        63 => new_cr.white_king_side = false,
        0 => new_cr.black_queen_side = false,
        7 => new_cr.black_king_side = false,
        _ => {}
    }

    (new_board, new_cr, new_ep)
}

/// Generate all legal moves (filters out moves that leave own king in check).
pub fn generate_legal_moves(
    board: &[Option<ChessPiece>; 64],
    color: Color,
    castling_rights: &CastlingRights,
    en_passant: Option<usize>,
) -> Vec<ChessMove> {
    let pseudo = generate_pseudo_legal_moves(board, color, castling_rights, en_passant);
    pseudo
        .into_iter()
        .filter(|mv| {
            let (new_board, _, _) = apply_move(board, mv, castling_rights, en_passant);
            !is_in_check(&new_board, color)
        })
        .collect()
}

pub fn is_checkmate(
    board: &[Option<ChessPiece>; 64],
    color: Color,
    cr: &CastlingRights,
    ep: Option<usize>,
) -> bool {
    is_in_check(board, color) && generate_legal_moves(board, color, cr, ep).is_empty()
}

pub fn is_stalemate(
    board: &[Option<ChessPiece>; 64],
    color: Color,
    cr: &CastlingRights,
    ep: Option<usize>,
) -> bool {
    !is_in_check(board, color) && generate_legal_moves(board, color, cr, ep).is_empty()
}
