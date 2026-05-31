use crate::moves::info::MoveInfo;
use crate::position::Position;
use crate::utils::Square;

pub struct GameState {
    pub position: Position,
    move_history: Vec<MoveInfo>,
    redo_history: Vec<MoveInfo>,
}

impl GameState {
    pub fn new(fen: &str) -> GameState {
        GameState {
            position: Position::new(fen),
            move_history: Vec::new(),
            redo_history: Vec::new(),
        }
    }

    pub fn move_history(&self) -> &[MoveInfo] {
        &self.move_history
    }

    pub fn redo_history(&self) -> &[MoveInfo] {
        &self.redo_history
    }

    pub fn last_move(&self) -> MoveInfo {
        self.move_history.last().cloned().unwrap_or_default()
    }

    pub fn play_move(&mut self, from: Square, to: Square) -> Option<MoveInfo> {
        let move_info = self.position.play_validated_move(from, to)?;
        self.move_history.push(move_info.clone());
        self.redo_history.clear();
        Some(move_info)
    }

    pub fn undo_move(&mut self) -> bool {
        if let Some(last_move) = self.move_history.pop() {
            self.position.undo_move(last_move.clone());
            self.redo_history.push(last_move);
            true
        } else {
            false
        }
    }

    pub fn redo_move(&mut self) -> bool {
        if let Some(last_move) = self.redo_history.pop() {
            if let Some(move_info) = self.position.play_validated_move(last_move.from, last_move.to) {
                self.move_history.push(move_info);
                true
            } else {
                self.redo_history.push(last_move);
                false
            }
        } else {
            false
        }
    }
}
