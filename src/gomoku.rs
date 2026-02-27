use crate::state::Player;

pub fn check_gomoku_win(board: &[Option<Player>; 225], player: Player) -> bool {
    let size = 15;
    for y in 0..size {
        for x in 0..size {
            let idx = y * size + x;
            if board[idx] != Some(player) {
                continue;
            }

            if x <= size - 5 && (1..5).all(|i| board[idx + i] == Some(player)) {
                return true;
            }
            if y <= size - 5 && (1..5).all(|i| board[idx + i * size] == Some(player)) {
                return true;
            }
            if x <= size - 5
                && y <= size - 5
                && (1..5).all(|i| board[idx + i * size + i] == Some(player))
            {
                return true;
            }
            if x >= 4
                && y <= size - 5
                && (1..5).all(|i| board[idx + i * size - i] == Some(player))
            {
                return true;
            }
        }
    }
    false
}

fn evaluate_window(ai_count: i32, human_count: i32) -> i32 {
    if ai_count > 0 && human_count == 0 {
        match ai_count {
            5 => 10000000,
            4 => 100000,
            3 => 1000,
            2 => 10,
            1 => 1,
            _ => 0,
        }
    } else if human_count > 0 && ai_count == 0 {
        match human_count {
            5 => -10000000,
            4 => -1000000,
            3 => -10000,
            2 => -10,
            1 => -1,
            _ => 0,
        }
    } else {
        0
    }
}

fn evaluate_board(board: &[Option<Player>; 225]) -> i32 {
    let mut score = 0;
    let size = 15;

    let mut score_window = |indices: [usize; 5]| {
        let mut ai_count = 0;
        let mut human_count = 0;
        for &idx in &indices {
            match board[idx] {
                Some(Player::O) => ai_count += 1,
                Some(Player::X) => human_count += 1,
                None => {}
            }
        }
        score += evaluate_window(ai_count, human_count);
    };

    for y in 0..size {
        for x in 0..size {
            let idx = y * size + x;
            if x <= size - 5 {
                score_window([idx, idx + 1, idx + 2, idx + 3, idx + 4]);
            }
            if y <= size - 5 {
                score_window([
                    idx,
                    idx + size,
                    idx + size * 2,
                    idx + size * 3,
                    idx + size * 4,
                ]);
            }
            if x <= size - 5 && y <= size - 5 {
                score_window([
                    idx,
                    idx + size + 1,
                    idx + (size + 1) * 2,
                    idx + (size + 1) * 3,
                    idx + (size + 1) * 4,
                ]);
            }
            if x >= 4 && y <= size - 5 {
                score_window([
                    idx,
                    idx + size - 1,
                    idx + (size - 1) * 2,
                    idx + (size - 1) * 3,
                    idx + (size - 1) * 4,
                ]);
            }
        }
    }
    score
}

fn get_relevant_moves(board: &[Option<Player>; 225]) -> Vec<usize> {
    let mut moves = Vec::new();
    let mut is_empty = true;

    for i in 0..225 {
        if board[i].is_some() {
            is_empty = false;
            continue;
        }

        let ix = (i % 15) as isize;
        let iy = (i / 15) as isize;
        let mut has_neighbor = false;

        'neighbor_search: for dy in -2..=2 {
            for dx in -2..=2 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = ix + dx;
                let ny = iy + dy;
                if (0..15).contains(&nx)
                    && (0..15).contains(&ny)
                    && board[(ny * 15 + nx) as usize].is_some()
                {
                    has_neighbor = true;
                    break 'neighbor_search;
                }
            }
        }
        if has_neighbor {
            moves.push(i);
        }
    }

    if is_empty { vec![112] } else { moves }
}

fn minimax_gomoku(
    board: &mut [Option<Player>; 225],
    depth: i32,
    mut alpha: i32,
    mut beta: i32,
    is_maximizing: bool,
) -> i32 {
    if check_gomoku_win(board, Player::O) {
        return 10000000;
    }
    if check_gomoku_win(board, Player::X) {
        return -10000000;
    }
    if depth == 0 || board.iter().all(|c| c.is_some()) {
        return evaluate_board(board);
    }

    let moves = get_relevant_moves(board);

    if is_maximizing {
        let mut max_eval = i32::MIN;
        for m in moves {
            board[m] = Some(Player::O);
            let eval = minimax_gomoku(board, depth - 1, alpha, beta, false);
            board[m] = None;
            max_eval = max_eval.max(eval);
            alpha = alpha.max(eval);
            if beta <= alpha {
                break;
            }
        }
        max_eval
    } else {
        let mut min_eval = i32::MAX;
        for m in moves {
            board[m] = Some(Player::X);
            let eval = minimax_gomoku(board, depth - 1, alpha, beta, true);
            board[m] = None;
            min_eval = min_eval.min(eval);
            beta = beta.min(eval);
            if beta <= alpha {
                break;
            }
        }
        min_eval
    }
}

pub fn get_best_gomoku_move(board: &[Option<Player>; 225]) -> Option<usize> {
    let mut cloned_board = *board;
    let moves = get_relevant_moves(&cloned_board);

    let mut best_score = i32::MIN;
    let mut best_move = None;
    let depth = 3;

    for m in moves {
        cloned_board[m] = Some(Player::O);
        let score = minimax_gomoku(&mut cloned_board, depth - 1, i32::MIN, i32::MAX, false);
        cloned_board[m] = None;

        if score > best_score {
            best_score = score;
            best_move = Some(m);
        }
    }

    best_move
}
