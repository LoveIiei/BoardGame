use super::{PieceType, XiangqiColor, XiangqiPiece};

#[derive(Copy, Clone, Debug)]
pub struct XiangqiMove {
    pub from: usize,
    pub to: usize,
}

fn in_bounds(row: isize, col: isize) -> bool {
    (0..10).contains(&row) && (0..9).contains(&col)
}

fn idx(row: isize, col: isize) -> usize {
    row as usize * 9 + col as usize
}

fn in_palace(row: usize, col: usize, color: XiangqiColor) -> bool {
    (3..=5).contains(&col)
        && match color {
            XiangqiColor::Red => (7..=9).contains(&row),
            XiangqiColor::Black => (0..=2).contains(&row),
        }
}

fn on_own_side(row: usize, color: XiangqiColor) -> bool {
    match color {
        XiangqiColor::Red => row >= 5,
        XiangqiColor::Black => row <= 4,
    }
}

fn crossed_river(row: usize, color: XiangqiColor) -> bool {
    match color {
        XiangqiColor::Red => row <= 4,
        XiangqiColor::Black => row >= 5,
    }
}

fn forward_dir(color: XiangqiColor) -> isize {
    match color {
        XiangqiColor::Red => -1,   // Red moves upward (decreasing row)
        XiangqiColor::Black => 1,  // Black moves downward (increasing row)
    }
}

fn find_general(board: &[Option<XiangqiPiece>; 90], color: XiangqiColor) -> usize {
    board
        .iter()
        .position(|p| {
            matches!(p, Some(piece) if piece.color == color && piece.piece_type == PieceType::General)
        })
        .expect("General must exist on board")
}

/// Check if the flying general rule is violated (two generals face each other
/// on the same column with no pieces between them).
fn is_flying_general(board: &[Option<XiangqiPiece>; 90]) -> bool {
    let red_gen = find_general(board, XiangqiColor::Red);
    let black_gen = find_general(board, XiangqiColor::Black);

    let red_col = red_gen % 9;
    let black_col = black_gen % 9;

    if red_col != black_col {
        return false;
    }

    let red_row = red_gen / 9;
    let black_row = black_gen / 9;
    let min_row = black_row.min(red_row);
    let max_row = black_row.max(red_row);

    for row in (min_row + 1)..max_row {
        if board[row * 9 + red_col].is_some() {
            return false;
        }
    }

    true
}

/// Check if the given color's general is under attack.
pub fn is_in_check(board: &[Option<XiangqiPiece>; 90], color: XiangqiColor) -> bool {
    let gen_sq = find_general(board, color);
    is_square_attacked(board, gen_sq, color.opposite())
}

fn is_square_attacked(
    board: &[Option<XiangqiPiece>; 90],
    square: usize,
    by_color: XiangqiColor,
) -> bool {
    let row = square / 9;
    let col = square % 9;

    for sq in 0..90 {
        let piece = match board[sq] {
            Some(p) if p.color == by_color => p,
            _ => continue,
        };

        let pr = sq / 9;
        let pc = sq % 9;

        match piece.piece_type {
            PieceType::General => {
                let dr = (row as isize - pr as isize).abs();
                let dc = (col as isize - pc as isize).abs();
                if (dr == 1 && dc == 0) || (dr == 0 && dc == 1) {
                    return true;
                }
            }
            PieceType::Advisor => {
                let dr = (row as isize - pr as isize).abs();
                let dc = (col as isize - pc as isize).abs();
                if dr == 1 && dc == 1 {
                    return true;
                }
            }
            PieceType::Elephant => {
                let dr = row as isize - pr as isize;
                let dc = col as isize - pc as isize;
                if dr.abs() == 2 && dc.abs() == 2 {
                    let mid_r = pr as isize + dr / 2;
                    let mid_c = pc as isize + dc / 2;
                    if board[idx(mid_r, mid_c)].is_none() {
                        return true;
                    }
                }
            }
            PieceType::Horse => {
                let dr = row as isize - pr as isize;
                let dc = col as isize - pc as isize;
                let is_horse_move =
                    (dr.abs() == 2 && dc.abs() == 1) || (dr.abs() == 1 && dc.abs() == 2);
                if is_horse_move {
                    // Check "horse leg" blocking at orthogonal midpoint
                    let (block_r, block_c) = if dr.abs() == 2 {
                        (pr as isize + dr / 2, pc as isize)
                    } else {
                        (pr as isize, pc as isize + dc / 2)
                    };
                    if board[idx(block_r, block_c)].is_none() {
                        return true;
                    }
                }
            }
            PieceType::Chariot => {
                if pr == row {
                    let min_c = col.min(pc);
                    let max_c = col.max(pc);
                    let mut blocked = false;
                    for c in (min_c + 1)..max_c {
                        if board[pr * 9 + c].is_some() {
                            blocked = true;
                            break;
                        }
                    }
                    if !blocked && min_c != max_c {
                        return true;
                    }
                } else if pc == col {
                    let min_r = row.min(pr);
                    let max_r = row.max(pr);
                    let mut blocked = false;
                    for r in (min_r + 1)..max_r {
                        if board[r * 9 + col].is_some() {
                            blocked = true;
                            break;
                        }
                    }
                    if !blocked && min_r != max_r {
                        return true;
                    }
                }
            }
            PieceType::Cannon => {
                if pr == row && pc != col {
                    let min_c = col.min(pc);
                    let max_c = col.max(pc);
                    let mut count = 0;
                    for c in (min_c + 1)..max_c {
                        if board[pr * 9 + c].is_some() {
                            count += 1;
                        }
                    }
                    if count == 1 {
                        return true;
                    }
                } else if pc == col && pr != row {
                    let min_r = row.min(pr);
                    let max_r = row.max(pr);
                    let mut count = 0;
                    for r in (min_r + 1)..max_r {
                        if board[r * 9 + col].is_some() {
                            count += 1;
                        }
                    }
                    if count == 1 {
                        return true;
                    }
                }
            }
            PieceType::Soldier => {
                let dir = forward_dir(by_color);
                // Forward attack
                if row as isize == pr as isize + dir && col == pc {
                    return true;
                }
                // Sideways attack (after crossing river)
                if crossed_river(pr, by_color)
                    && row == pr
                    && (col as isize - pc as isize).abs() == 1
                {
                    return true;
                }
            }
        }
    }

    false
}

fn generate_pseudo_legal_moves(
    board: &[Option<XiangqiPiece>; 90],
    color: XiangqiColor,
) -> Vec<XiangqiMove> {
    let mut moves = Vec::new();

    for sq in 0..90 {
        let piece = match board[sq] {
            Some(p) if p.color == color => p,
            _ => continue,
        };

        let row = sq / 9;
        let col = sq % 9;

        match piece.piece_type {
            PieceType::General => {
                let dirs: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
                for (dr, dc) in dirs {
                    let nr = row as isize + dr;
                    let nc = col as isize + dc;
                    if in_bounds(nr, nc) && in_palace(nr as usize, nc as usize, color) {
                        let to = idx(nr, nc);
                        if board[to].is_none_or(|p| p.color != color) {
                            moves.push(XiangqiMove { from: sq, to });
                        }
                    }
                }
            }
            PieceType::Advisor => {
                let dirs: [(isize, isize); 4] = [(-1, -1), (-1, 1), (1, -1), (1, 1)];
                for (dr, dc) in dirs {
                    let nr = row as isize + dr;
                    let nc = col as isize + dc;
                    if in_bounds(nr, nc) && in_palace(nr as usize, nc as usize, color) {
                        let to = idx(nr, nc);
                        if board[to].is_none_or(|p| p.color != color) {
                            moves.push(XiangqiMove { from: sq, to });
                        }
                    }
                }
            }
            PieceType::Elephant => {
                let dirs: [(isize, isize); 4] = [(-2, -2), (-2, 2), (2, -2), (2, 2)];
                for (dr, dc) in dirs {
                    let nr = row as isize + dr;
                    let nc = col as isize + dc;
                    if in_bounds(nr, nc) && on_own_side(nr as usize, color) {
                        let mid_r = row as isize + dr / 2;
                        let mid_c = col as isize + dc / 2;
                        if board[idx(mid_r, mid_c)].is_none() {
                            let to = idx(nr, nc);
                            if board[to].is_none_or(|p| p.color != color) {
                                moves.push(XiangqiMove { from: sq, to });
                            }
                        }
                    }
                }
            }
            PieceType::Horse => {
                // (dr, dc, block_dr, block_dc)
                let offsets: [(isize, isize, isize, isize); 8] = [
                    (-2, -1, -1, 0),
                    (-2, 1, -1, 0),
                    (2, -1, 1, 0),
                    (2, 1, 1, 0),
                    (-1, -2, 0, -1),
                    (-1, 2, 0, 1),
                    (1, -2, 0, -1),
                    (1, 2, 0, 1),
                ];
                for (dr, dc, bdr, bdc) in offsets {
                    let nr = row as isize + dr;
                    let nc = col as isize + dc;
                    if in_bounds(nr, nc) {
                        let block_r = row as isize + bdr;
                        let block_c = col as isize + bdc;
                        if board[idx(block_r, block_c)].is_none() {
                            let to = idx(nr, nc);
                            if board[to].is_none_or(|p| p.color != color) {
                                moves.push(XiangqiMove { from: sq, to });
                            }
                        }
                    }
                }
            }
            PieceType::Chariot => {
                let dirs: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
                for (dr, dc) in dirs {
                    let mut nr = row as isize + dr;
                    let mut nc = col as isize + dc;
                    while in_bounds(nr, nc) {
                        let to = idx(nr, nc);
                        match board[to] {
                            None => moves.push(XiangqiMove { from: sq, to }),
                            Some(p) if p.color != color => {
                                moves.push(XiangqiMove { from: sq, to });
                                break;
                            }
                            _ => break,
                        }
                        nr += dr;
                        nc += dc;
                    }
                }
            }
            PieceType::Cannon => {
                let dirs: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
                for (dr, dc) in dirs {
                    let mut nr = row as isize + dr;
                    let mut nc = col as isize + dc;
                    // Non-capture: slide until blocked
                    while in_bounds(nr, nc) {
                        let to = idx(nr, nc);
                        if board[to].is_some() {
                            // Found platform piece — look for capture target beyond it
                            nr += dr;
                            nc += dc;
                            while in_bounds(nr, nc) {
                                let cap_to = idx(nr, nc);
                                if let Some(p) = board[cap_to] {
                                    if p.color != color {
                                        moves.push(XiangqiMove {
                                            from: sq,
                                            to: cap_to,
                                        });
                                    }
                                    break;
                                }
                                nr += dr;
                                nc += dc;
                            }
                            break;
                        }
                        moves.push(XiangqiMove { from: sq, to });
                        nr += dr;
                        nc += dc;
                    }
                }
            }
            PieceType::Soldier => {
                let dir = forward_dir(color);
                // Forward move
                let nr = row as isize + dir;
                if in_bounds(nr, col as isize) {
                    let to = idx(nr, col as isize);
                    if board[to].is_none_or(|p| p.color != color) {
                        moves.push(XiangqiMove { from: sq, to });
                    }
                }
                // Sideways moves (after crossing river)
                if crossed_river(row, color) {
                    for dc in [-1isize, 1] {
                        let nc = col as isize + dc;
                        if in_bounds(row as isize, nc) {
                            let to = idx(row as isize, nc);
                            if board[to].is_none_or(|p| p.color != color) {
                                moves.push(XiangqiMove { from: sq, to });
                            }
                        }
                    }
                }
            }
        }
    }

    moves
}

pub fn apply_move(
    board: &[Option<XiangqiPiece>; 90],
    mv: &XiangqiMove,
) -> [Option<XiangqiPiece>; 90] {
    let mut new_board = *board;
    new_board[mv.to] = new_board[mv.from];
    new_board[mv.from] = None;
    new_board
}

/// Generate all legal moves (filters out moves that leave own general in check
/// or create a flying general situation).
pub fn generate_legal_moves(
    board: &[Option<XiangqiPiece>; 90],
    color: XiangqiColor,
) -> Vec<XiangqiMove> {
    let pseudo = generate_pseudo_legal_moves(board, color);
    pseudo
        .into_iter()
        .filter(|mv| {
            let new_board = apply_move(board, mv);
            !is_in_check(&new_board, color) && !is_flying_general(&new_board)
        })
        .collect()
}

pub fn is_checkmate(board: &[Option<XiangqiPiece>; 90], color: XiangqiColor) -> bool {
    is_in_check(board, color) && generate_legal_moves(board, color).is_empty()
}

pub fn is_stalemate(board: &[Option<XiangqiPiece>; 90], color: XiangqiColor) -> bool {
    !is_in_check(board, color) && generate_legal_moves(board, color).is_empty()
}
