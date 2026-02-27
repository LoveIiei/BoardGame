use crate::state::Player;

const TTT_WIN_LINES: [[usize; 3]; 8] = [
    [0, 1, 2],
    [3, 4, 5],
    [6, 7, 8],
    [0, 3, 6],
    [1, 4, 7],
    [2, 5, 8],
    [0, 4, 8],
    [2, 4, 6],
];

pub fn check_ttt_win(board: &[Option<Player>; 9], player: Player) -> bool {
    TTT_WIN_LINES
        .iter()
        .any(|line| line.iter().all(|&idx| board[idx] == Some(player)))
}

fn minimax_ttt(board: &mut [Option<Player>; 9], depth: i32, is_maximizing: bool) -> i32 {
    if check_ttt_win(board, Player::O) {
        return 10 - depth;
    }
    if check_ttt_win(board, Player::X) {
        return depth - 10;
    }
    if board.iter().all(|c| c.is_some()) {
        return 0;
    }

    let mut best_score = if is_maximizing { i32::MIN } else { i32::MAX };
    for i in 0..9 {
        if board[i].is_none() {
            board[i] = Some(if is_maximizing { Player::O } else { Player::X });
            let score = minimax_ttt(board, depth + 1, !is_maximizing);
            board[i] = None;
            best_score = if is_maximizing {
                best_score.max(score)
            } else {
                best_score.min(score)
            };
        }
    }
    best_score
}

pub fn get_best_ttt_move(board: &mut [Option<Player>; 9]) -> Option<usize> {
    let mut best_score = i32::MIN;
    let mut best_move = None;
    for i in 0..9 {
        if board[i].is_none() {
            board[i] = Some(Player::O);
            let score = minimax_ttt(board, 0, false);
            board[i] = None;
            if score > best_score {
                best_score = score;
                best_move = Some(i);
            }
        }
    }
    best_move
}
