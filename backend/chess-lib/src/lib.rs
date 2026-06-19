#![allow(dead_code)]

/// A bitboard representation of a chessboard
pub struct Chessboard {
    bitboard: u64,
}

// This will be here for now methinks
#[allow(clippy::derivable_impls)]
impl Default for Chessboard {
    fn default() -> Self {
        Self {
            bitboard: 0, // An empty board where all bits = 0
        }
    }
}

enum Color {
    White,
    Black,
}

enum PieceType {
    King,
    Knight,
    Bishop,
    Rook,
    Queen,
    Pawn,
}

struct Piece {
    color: Color,
    kind: PieceType,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_board() {
        let board = Chessboard::default();
        assert_eq!(board.bitboard, 0);
    }
}
