slint::include_modules!();

mod chess;
mod gomoku;
mod state;
mod ttt;
mod xiangqi;

use chess::moves::{apply_move, generate_legal_moves, is_checkmate, is_in_check, is_stalemate};
use chess::{CastlingRights, ChessPiece, Color, piece_to_unicode};
use slint::{SharedString, VecModel};
use state::{AppState, GameMode, GameType, Player};
use std::io::BufReader;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use rodio::{Decoder, OutputStream, Sink, Source};

/// Returned by chess click handler to tell the caller what to do after releasing the lock.
enum ChessAction {
    /// Just update the UI (selection changed, move executed without AI)
    UpdateOnly,
    /// Spawn the AI thread with the given board state
    SpawnAI {
        board: [Option<ChessPiece>; 64],
        cr: CastlingRights,
        ep: Option<usize>,
    },
}

/// Returned by xiangqi click handler to tell the caller what to do after releasing the lock.
enum XiangqiAction {
    UpdateOnly,
    SpawnAI {
        board: [Option<xiangqi::XiangqiPiece>; 90],
    },
}

fn main() -> Result<(), slint::PlatformError> {
    // Initialize audio (graceful: no crash if no audio device or missing file)
    let _audio_stream = OutputStream::try_default().ok();
    let music_sink: Option<Sink> = _audio_stream.as_ref().and_then(|(_, handle)| {
        let file = std::fs::File::open("assets/bgm.mp3").ok()?;
        let source = Decoder::new(BufReader::new(file)).ok()?.repeat_infinite();
        let sink = Sink::try_new(handle).ok()?;
        sink.append(source);
        Some(sink)
    });
    let music_sink = Arc::new(music_sink);

    let ui = AppWindow::new()?;
    let ui_handle = ui.as_weak();
    let state = Arc::new(Mutex::new(AppState::new()));

    // Set initial music state
    ui.set_music_playing(music_sink.is_some());

    // Helper: Sync Rust arrays to Slint UI
    let update_ui = {
        let ui_handle = ui_handle.clone();
        let state = state.clone();
        move || {
            if let Some(ui) = ui_handle.upgrade() {
                let st = state.lock().unwrap();

                // Sync TTT
                let mut ttt_vec: Vec<SharedString> = vec![];
                for cell in st.ttt_board.iter() {
                    ttt_vec.push(match cell {
                        Some(Player::X) => "X".into(),
                        Some(Player::O) => "O".into(),
                        None => "".into(),
                    });
                }
                ui.set_ttt_board(Rc::new(VecModel::from(ttt_vec)).into());

                // Sync Gomoku
                let mut gmk_vec: Vec<SharedString> = vec![];
                for cell in st.gomoku_board.iter() {
                    gmk_vec.push(match cell {
                        Some(Player::X) => "X".into(),
                        Some(Player::O) => "O".into(),
                        None => "".into(),
                    });
                }
                ui.set_gomoku_board(Rc::new(VecModel::from(gmk_vec)).into());

                // Sync last move (TTT/Gomoku)
                ui.set_last_move(st.last_move.map_or(-1, |m| m as i32));

                // Sync Chess
                let mut chess_vec: Vec<ChessCell> = vec![];
                for (i, cell) in st.chess_board.iter().enumerate() {
                    let piece_str: SharedString = match cell {
                        Some(p) => piece_to_unicode(p).into(),
                        None => "".into(),
                    };
                    chess_vec.push(ChessCell {
                        piece: piece_str,
                        is_valid_move: st.valid_moves.contains(&i),
                        is_selected: st.selected_cell == Some(i),
                    });
                }
                ui.set_chess_board(Rc::new(VecModel::from(chess_vec)).into());
                ui.set_chess_last_move_from(st.last_move_from.map_or(-1, |m| m as i32));
                ui.set_chess_last_move_to(st.last_move_to.map_or(-1, |m| m as i32));

                // Sync Xiangqi
                let mut xq_vec: Vec<XiangqiCell> = vec![];
                for (i, cell) in st.xiangqi_board.iter().enumerate() {
                    let (piece_str, piece_color): (SharedString, SharedString) = match cell {
                        Some(p) => (
                            xiangqi::piece_to_text(p).into(),
                            match p.color {
                                xiangqi::XiangqiColor::Red => "red".into(),
                                xiangqi::XiangqiColor::Black => "black".into(),
                            },
                        ),
                        None => ("".into(), "".into()),
                    };
                    xq_vec.push(XiangqiCell {
                        piece: piece_str,
                        piece_color,
                        is_valid_move: st.xiangqi_valid_moves.contains(&i),
                        is_selected: st.xiangqi_selected == Some(i),
                    });
                }
                ui.set_xiangqi_board(Rc::new(VecModel::from(xq_vec)).into());
                ui.set_xiangqi_last_move_from(st.xiangqi_last_from.map_or(-1, |m| m as i32));
                ui.set_xiangqi_last_move_to(st.xiangqi_last_to.map_or(-1, |m| m as i32));
            }
        }
    };

    // Initialize boards on startup
    update_ui();

    // Menu Routing
    {
        let ui_handle = ui_handle.clone();
        let state = state.clone();
        let update_ui = update_ui.clone();
        ui.on_start_game(move |game_type, game_mode| {
            let mut st = state.lock().unwrap();
            *st = AppState::new();

            st.game_type = match game_type.as_str() {
                "ttt" => GameType::TicTacToe,
                "gomoku" => GameType::Gomoku,
                "chess" => GameType::Chess,
                "xiangqi" => GameType::Xiangqi,
                _ => GameType::None,
            };
            st.game_mode = if game_mode == "pvp" {
                GameMode::PvP
            } else {
                GameMode::PvE
            };

            if let Some(ui) = ui_handle.upgrade() {
                ui.set_current_page(game_type);
                let status = match st.game_type {
                    GameType::Chess => {
                        if st.game_mode == GameMode::PvP {
                            "White's Turn"
                        } else {
                            "Your Turn (White)"
                        }
                    }
                    GameType::Xiangqi => {
                        if st.game_mode == GameMode::PvP {
                            "Red's Turn"
                        } else {
                            "Your Turn (Red)"
                        }
                    }
                    _ => {
                        if st.game_mode == GameMode::PvP {
                            "Player X's Turn"
                        } else {
                            "Your Turn (X)"
                        }
                    }
                };
                ui.set_status_text(status.into());
            }
            drop(st);
            update_ui();
        });
    }

    // Return to Menu
    {
        let ui_handle = ui_handle.clone();
        ui.on_return_to_menu(move || {
            if let Some(ui) = ui_handle.upgrade() {
                ui.set_current_page("menu".into());
            }
        });
    }

    // Handle Clicks
    {
        let ui_handle = ui_handle.clone();
        let state = state.clone();
        ui.on_cell_clicked(move |index| {
            let idx = index as usize;
            let mut st = state.lock().unwrap();

            if st.game_over || st.ai_thinking {
                return;
            }

            // === TIC-TAC-TOE ===
            if st.game_type == GameType::TicTacToe {
                if st.ttt_board[idx].is_some() {
                    return;
                }

                if st.game_mode == GameMode::PvP {
                    st.ttt_board[idx] = Some(st.current_turn);
                    st.last_move = Some(idx);
                    if ttt::check_ttt_win(&st.ttt_board, st.current_turn) {
                        if let Some(ui) = ui_handle.upgrade() {
                            ui.set_status_text(
                                format!("Player {:?} Wins!", st.current_turn).into(),
                            );
                        }
                        st.game_over = true;
                    } else if st.ttt_board.iter().all(|c| c.is_some()) {
                        if let Some(ui) = ui_handle.upgrade() {
                            ui.set_status_text("It's a Draw!".into());
                        }
                        st.game_over = true;
                    } else {
                        st.current_turn = if st.current_turn == Player::X {
                            Player::O
                        } else {
                            Player::X
                        };
                        if let Some(ui) = ui_handle.upgrade() {
                            ui.set_status_text(
                                format!("Player {:?}'s Turn", st.current_turn).into(),
                            );
                        }
                    }
                    drop(st);
                    update_ui();
                } else {
                    // PvE
                    st.ttt_board[idx] = Some(Player::X);
                    st.last_move = Some(idx);
                    if ttt::check_ttt_win(&st.ttt_board, Player::X) {
                        if let Some(ui) = ui_handle.upgrade() {
                            ui.set_status_text("You Win!".into());
                        }
                        st.game_over = true;
                        drop(st);
                        update_ui();
                    } else if st.ttt_board.iter().all(|c| c.is_some()) {
                        if let Some(ui) = ui_handle.upgrade() {
                            ui.set_status_text("It's a Draw!".into());
                        }
                        st.game_over = true;
                        drop(st);
                        update_ui();
                    } else {
                        st.ai_thinking = true;
                        let board_copy = st.ttt_board;
                        drop(st);
                        update_ui();

                        if let Some(ui) = ui_handle.upgrade() {
                            ui.set_status_text("Thinking...".into());
                        }

                        let state = state.clone();
                        let ui_handle = ui_handle.clone();
                        let update_ui = update_ui.clone();
                        std::thread::spawn(move || {
                            let best_move = ttt::get_best_ttt_move(&mut board_copy.clone());
                            slint::invoke_from_event_loop(move || {
                                let mut st = state.lock().unwrap();
                                if !st.ai_thinking {
                                    return;
                                }
                                if let Some(m) = best_move {
                                    st.ttt_board[m] = Some(Player::O);
                                    st.last_move = Some(m);
                                    if ttt::check_ttt_win(&st.ttt_board, Player::O) {
                                        if let Some(ui) = ui_handle.upgrade() {
                                            ui.set_status_text("Computer Wins!".into());
                                        }
                                        st.game_over = true;
                                    } else if st.ttt_board.iter().all(|c| c.is_some()) {
                                        if let Some(ui) = ui_handle.upgrade() {
                                            ui.set_status_text("It's a Draw!".into());
                                        }
                                        st.game_over = true;
                                    } else if let Some(ui) = ui_handle.upgrade() {
                                        ui.set_status_text("Your Turn (X)".into());
                                    }
                                }
                                st.ai_thinking = false;
                                drop(st);
                                update_ui();
                            })
                            .unwrap();
                        });
                    }
                }
            }
            // === GOMOKU ===
            else if st.game_type == GameType::Gomoku {
                if st.gomoku_board[idx].is_some() {
                    return;
                }

                if st.game_mode == GameMode::PvP {
                    st.gomoku_board[idx] = Some(st.current_turn);
                    st.last_move = Some(idx);
                    if gomoku::check_gomoku_win(&st.gomoku_board, st.current_turn) {
                        if let Some(ui) = ui_handle.upgrade() {
                            ui.set_status_text(
                                format!("Player {:?} Wins!", st.current_turn).into(),
                            );
                        }
                        st.game_over = true;
                    } else {
                        st.current_turn = if st.current_turn == Player::X {
                            Player::O
                        } else {
                            Player::X
                        };
                        if let Some(ui) = ui_handle.upgrade() {
                            ui.set_status_text(
                                format!("Player {:?}'s Turn", st.current_turn).into(),
                            );
                        }
                    }
                    drop(st);
                    update_ui();
                } else {
                    // PvE
                    st.gomoku_board[idx] = Some(Player::X);
                    st.last_move = Some(idx);
                    if gomoku::check_gomoku_win(&st.gomoku_board, Player::X) {
                        if let Some(ui) = ui_handle.upgrade() {
                            ui.set_status_text("You Win!".into());
                        }
                        st.game_over = true;
                        drop(st);
                        update_ui();
                    } else {
                        st.ai_thinking = true;
                        let board_copy = st.gomoku_board;
                        drop(st);
                        update_ui();

                        if let Some(ui) = ui_handle.upgrade() {
                            ui.set_status_text("Thinking...".into());
                        }

                        let state = state.clone();
                        let ui_handle = ui_handle.clone();
                        let update_ui = update_ui.clone();
                        std::thread::spawn(move || {
                            let best_move = gomoku::get_best_gomoku_move(&board_copy);
                            slint::invoke_from_event_loop(move || {
                                let mut st = state.lock().unwrap();
                                if !st.ai_thinking {
                                    return;
                                }
                                if let Some(m) = best_move {
                                    st.gomoku_board[m] = Some(Player::O);
                                    st.last_move = Some(m);
                                    if gomoku::check_gomoku_win(&st.gomoku_board, Player::O) {
                                        if let Some(ui) = ui_handle.upgrade() {
                                            ui.set_status_text("Computer Wins!".into());
                                        }
                                        st.game_over = true;
                                    } else if let Some(ui) = ui_handle.upgrade() {
                                        ui.set_status_text("Your Turn (X)".into());
                                    }
                                }
                                st.ai_thinking = false;
                                drop(st);
                                update_ui();
                            })
                            .unwrap();
                        });
                    }
                }
            }
            // === CHESS ===
            else if st.game_type == GameType::Chess {
                let action = handle_chess_click(&mut st, idx, &ui_handle);
                match action {
                    ChessAction::UpdateOnly => {
                        drop(st);
                        update_ui();
                    }
                    ChessAction::SpawnAI { board, cr, ep } => {
                        drop(st);
                        update_ui();

                        let state = state.clone();
                        let ui_handle = ui_handle.clone();
                        let update_ui = update_ui.clone();
                        std::thread::spawn(move || {
                            let best_move =
                                chess::ai::get_best_chess_move(&board, Color::Black, &cr, ep);
                            slint::invoke_from_event_loop(move || {
                                let mut st = state.lock().unwrap();
                                if !st.ai_thinking {
                                    return;
                                }
                                if let Some(mv) = best_move {
                                    let (new_board, new_cr, new_ep) = apply_move(
                                        &st.chess_board,
                                        &mv,
                                        &st.castling_rights,
                                        st.en_passant_target,
                                    );
                                    st.chess_board = new_board;
                                    st.castling_rights = new_cr;
                                    st.en_passant_target = new_ep;
                                    st.last_move_from = Some(mv.from);
                                    st.last_move_to = Some(mv.to);
                                    st.chess_turn = Color::White;

                                    if is_checkmate(
                                        &st.chess_board,
                                        Color::White,
                                        &st.castling_rights,
                                        st.en_passant_target,
                                    ) {
                                        if let Some(ui) = ui_handle.upgrade() {
                                            ui.set_status_text("Black Wins!".into());
                                        }
                                        st.game_over = true;
                                    } else if is_stalemate(
                                        &st.chess_board,
                                        Color::White,
                                        &st.castling_rights,
                                        st.en_passant_target,
                                    ) {
                                        if let Some(ui) = ui_handle.upgrade() {
                                            ui.set_status_text("Draw!".into());
                                        }
                                        st.game_over = true;
                                    } else if is_in_check(&st.chess_board, Color::White) {
                                        if let Some(ui) = ui_handle.upgrade() {
                                            ui.set_status_text("White is in Check!".into());
                                        }
                                    } else if let Some(ui) = ui_handle.upgrade() {
                                        ui.set_status_text("Your Turn (White)".into());
                                    }
                                }
                                st.ai_thinking = false;
                                drop(st);
                                update_ui();
                            })
                            .unwrap();
                        });
                    }
                }
            }
            // === XIANGQI ===
            else if st.game_type == GameType::Xiangqi {
                let action = handle_xiangqi_click(&mut st, idx, &ui_handle);
                match action {
                    XiangqiAction::UpdateOnly => {
                        drop(st);
                        update_ui();
                    }
                    XiangqiAction::SpawnAI { board } => {
                        drop(st);
                        update_ui();

                        let state = state.clone();
                        let ui_handle = ui_handle.clone();
                        let update_ui = update_ui.clone();
                        std::thread::spawn(move || {
                            let best_move = xiangqi::ai::get_best_xiangqi_move(
                                &board,
                                xiangqi::XiangqiColor::Black,
                            );
                            slint::invoke_from_event_loop(move || {
                                let mut st = state.lock().unwrap();
                                if !st.ai_thinking {
                                    return;
                                }
                                if let Some(mv) = best_move {
                                    let new_board =
                                        xiangqi::moves::apply_move(&st.xiangqi_board, &mv);
                                    st.xiangqi_board = new_board;
                                    st.xiangqi_last_from = Some(mv.from);
                                    st.xiangqi_last_to = Some(mv.to);
                                    st.xiangqi_turn = xiangqi::XiangqiColor::Red;

                                    if xiangqi::moves::is_checkmate(
                                        &st.xiangqi_board,
                                        xiangqi::XiangqiColor::Red,
                                    ) || xiangqi::moves::is_stalemate(
                                        &st.xiangqi_board,
                                        xiangqi::XiangqiColor::Red,
                                    ) {
                                        if let Some(ui) = ui_handle.upgrade() {
                                            ui.set_status_text("Black Wins!".into());
                                        }
                                        st.game_over = true;
                                    } else if xiangqi::moves::is_in_check(
                                        &st.xiangqi_board,
                                        xiangqi::XiangqiColor::Red,
                                    ) {
                                        if let Some(ui) = ui_handle.upgrade() {
                                            ui.set_status_text("Red is in Check!".into());
                                        }
                                    } else if let Some(ui) = ui_handle.upgrade() {
                                        ui.set_status_text("Your Turn (Red)".into());
                                    }
                                }
                                st.ai_thinking = false;
                                drop(st);
                                update_ui();
                            })
                            .unwrap();
                        });
                    }
                }
            }
        });
    }

    // Music toggle
    {
        let music_sink = music_sink.clone();
        let ui_handle = ui_handle.clone();
        ui.on_toggle_music(move || {
            if let Some(sink) = music_sink.as_ref() {
                if sink.is_paused() {
                    sink.play();
                } else {
                    sink.pause();
                }
                if let Some(ui) = ui_handle.upgrade() {
                    ui.set_music_playing(!sink.is_paused());
                }
            }
        });
    }

    ui.run()
}

/// Handle a chess click. Returns a ChessAction telling the caller what to do after releasing the lock.
fn handle_chess_click(
    st: &mut AppState,
    idx: usize,
    ui_handle: &slint::Weak<AppWindow>,
) -> ChessAction {
    let clicked_piece = st.chess_board[idx];

    // Check if clicking on a valid move destination
    if st.selected_cell.is_some() && st.valid_moves.contains(&idx) {
        let from = st.selected_cell.unwrap();
        let legal_moves = generate_legal_moves(
            &st.chess_board,
            st.chess_turn,
            &st.castling_rights,
            st.en_passant_target,
        );
        if let Some(mv) = legal_moves.iter().find(|m| m.from == from && m.to == idx) {
            let (new_board, new_cr, new_ep) =
                apply_move(&st.chess_board, mv, &st.castling_rights, st.en_passant_target);
            st.chess_board = new_board;
            st.castling_rights = new_cr;
            st.en_passant_target = new_ep;
            st.last_move_from = Some(from);
            st.last_move_to = Some(idx);
            st.selected_cell = None;
            st.valid_moves.clear();

            let next_turn = st.chess_turn.opposite();
            st.chess_turn = next_turn;

            // Check game end conditions
            if is_checkmate(&st.chess_board, next_turn, &st.castling_rights, st.en_passant_target)
            {
                let winner_name = if next_turn == Color::White {
                    "Black"
                } else {
                    "White"
                };
                if let Some(ui) = ui_handle.upgrade() {
                    ui.set_status_text(format!("{winner_name} Wins!").into());
                }
                st.game_over = true;
                return ChessAction::UpdateOnly;
            }
            if is_stalemate(&st.chess_board, next_turn, &st.castling_rights, st.en_passant_target)
            {
                if let Some(ui) = ui_handle.upgrade() {
                    ui.set_status_text("Draw!".into());
                }
                st.game_over = true;
                return ChessAction::UpdateOnly;
            }
            if is_in_check(&st.chess_board, next_turn) {
                let in_check_name = if next_turn == Color::White {
                    "White"
                } else {
                    "Black"
                };
                if let Some(ui) = ui_handle.upgrade() {
                    ui.set_status_text(format!("{in_check_name} is in Check!").into());
                }
            } else if st.game_mode == GameMode::PvP {
                let turn_name = if next_turn == Color::White {
                    "White"
                } else {
                    "Black"
                };
                if let Some(ui) = ui_handle.upgrade() {
                    ui.set_status_text(format!("{turn_name}'s Turn").into());
                }
            } else if let Some(ui) = ui_handle.upgrade() {
                ui.set_status_text("Thinking...".into());
            }

            // If PvE and AI's turn, signal to spawn AI thread
            if st.game_mode == GameMode::PvE && next_turn == Color::Black {
                st.ai_thinking = true;
                return ChessAction::SpawnAI {
                    board: st.chess_board,
                    cr: st.castling_rights,
                    ep: st.en_passant_target,
                };
            }
        }
    } else if let Some(piece) = clicked_piece {
        if piece.color == st.chess_turn {
            // In PvE, only allow selecting White pieces
            if st.game_mode == GameMode::PvE && piece.color != Color::White {
                return ChessAction::UpdateOnly;
            }
            st.selected_cell = Some(idx);
            let legal_moves = generate_legal_moves(
                &st.chess_board,
                st.chess_turn,
                &st.castling_rights,
                st.en_passant_target,
            );
            st.valid_moves = legal_moves
                .iter()
                .filter(|m| m.from == idx)
                .map(|m| m.to)
                .collect();
        } else {
            st.selected_cell = None;
            st.valid_moves.clear();
        }
    } else {
        st.selected_cell = None;
        st.valid_moves.clear();
    }

    ChessAction::UpdateOnly
}

/// Handle a xiangqi click. Returns a XiangqiAction telling the caller what to do after releasing the lock.
fn handle_xiangqi_click(
    st: &mut AppState,
    idx: usize,
    ui_handle: &slint::Weak<AppWindow>,
) -> XiangqiAction {
    let clicked_piece = st.xiangqi_board[idx];

    // Check if clicking on a valid move destination
    if st.xiangqi_selected.is_some() && st.xiangqi_valid_moves.contains(&idx) {
        let from = st.xiangqi_selected.unwrap();
        let legal_moves =
            xiangqi::moves::generate_legal_moves(&st.xiangqi_board, st.xiangqi_turn);
        if let Some(mv) = legal_moves.iter().find(|m| m.from == from && m.to == idx) {
            let new_board = xiangqi::moves::apply_move(&st.xiangqi_board, mv);
            st.xiangqi_board = new_board;
            st.xiangqi_last_from = Some(from);
            st.xiangqi_last_to = Some(idx);
            st.xiangqi_selected = None;
            st.xiangqi_valid_moves.clear();

            let next_turn = st.xiangqi_turn.opposite();
            st.xiangqi_turn = next_turn;

            // Check game end conditions (in xiangqi, stalemate is also a loss)
            if xiangqi::moves::is_checkmate(&st.xiangqi_board, next_turn)
                || xiangqi::moves::is_stalemate(&st.xiangqi_board, next_turn)
            {
                let winner_name = if next_turn == xiangqi::XiangqiColor::Red {
                    "Black"
                } else {
                    "Red"
                };
                if let Some(ui) = ui_handle.upgrade() {
                    ui.set_status_text(format!("{winner_name} Wins!").into());
                }
                st.game_over = true;
                return XiangqiAction::UpdateOnly;
            }
            if xiangqi::moves::is_in_check(&st.xiangqi_board, next_turn) {
                let in_check_name = if next_turn == xiangqi::XiangqiColor::Red {
                    "Red"
                } else {
                    "Black"
                };
                if let Some(ui) = ui_handle.upgrade() {
                    ui.set_status_text(format!("{in_check_name} is in Check!").into());
                }
            } else if st.game_mode == GameMode::PvP {
                let turn_name = if next_turn == xiangqi::XiangqiColor::Red {
                    "Red"
                } else {
                    "Black"
                };
                if let Some(ui) = ui_handle.upgrade() {
                    ui.set_status_text(format!("{turn_name}'s Turn").into());
                }
            } else if let Some(ui) = ui_handle.upgrade() {
                ui.set_status_text("Thinking...".into());
            }

            // If PvE and AI's turn, signal to spawn AI thread
            if st.game_mode == GameMode::PvE && next_turn == xiangqi::XiangqiColor::Black {
                st.ai_thinking = true;
                return XiangqiAction::SpawnAI {
                    board: st.xiangqi_board,
                };
            }
        }
    } else if let Some(piece) = clicked_piece {
        if piece.color == st.xiangqi_turn {
            // In PvE, only allow selecting Red pieces
            if st.game_mode == GameMode::PvE && piece.color != xiangqi::XiangqiColor::Red {
                return XiangqiAction::UpdateOnly;
            }
            st.xiangqi_selected = Some(idx);
            let legal_moves =
                xiangqi::moves::generate_legal_moves(&st.xiangqi_board, st.xiangqi_turn);
            st.xiangqi_valid_moves = legal_moves
                .iter()
                .filter(|m| m.from == idx)
                .map(|m| m.to)
                .collect();
        } else {
            st.xiangqi_selected = None;
            st.xiangqi_valid_moves.clear();
        }
    } else {
        st.xiangqi_selected = None;
        st.xiangqi_valid_moves.clear();
    }

    XiangqiAction::UpdateOnly
}
