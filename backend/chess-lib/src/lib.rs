#![allow(dead_code)]

use std::fmt::Display;

/// A bitboard representation of a chessboard
#[derive(Default)]
pub struct Bitboard {
    bitboard: u64,
}

impl Bitboard {
    pub fn new(bitboard: u64) -> Self {
        Self { bitboard }
    }
}

impl Display for Bitboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // We print rank 8 first, then rank 7, and so on. The convention with
        // bitboards is that the leftmost digit (most significant, 63rd bit)
        // represents h8, and the righmost digit (least significant, 1st bit)
        // represents a1.
        for rank in (0..8).rev() {
            // We don't reverse this since we want to print from a to h, not h
            // to a
            for file in 0..8 {
                let square = rank * 8 + file;
                let mask = 1u64 << square;

                // If the bit is not equal to 1, the bit must be 0
                let bit = ((self.bitboard & mask) != 0) as u8;

                write!(f, "{}", bit)?;
            }
            writeln!(f)?;
        }

        Ok(())
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
    use indoc::indoc;

    use super::*;

    #[test]
    fn empty_board() {
        let board = Bitboard::default();
        let expected = indoc! {r#"
            00000000
            00000000
            00000000
            00000000
            00000000
            00000000
            00000000
            00000000
        "#};

        assert_eq!(board.to_string(), expected);
    }

    #[test]
    fn single_piece() {
        let board = Bitboard::new(268_435_456); // Place single piece on e4
        let expected = indoc! {r#"
            00000000
            00000000
            00000000
            00000000
            00001000
            00000000
            00000000
            00000000
        "#};

        assert_eq!(board.to_string(), expected);
    }
}
