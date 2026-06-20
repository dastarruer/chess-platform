#![allow(dead_code)]

use std::{
    fmt::Display,
    ops::{BitOr, BitOrAssign},
};

use strum::EnumCount;

pub struct Chessboard {
    pieces: [[Bitboard; PieceType::COUNT]; Side::COUNT],
}

impl Chessboard {
    /// Get all squares occupied by pieces
    fn occupied(&self) -> Bitboard {
        let mut board = Bitboard::new(0);

        for piece_board in self.pieces.as_flattened() {
            board |= *piece_board;
        }

        board
    }
}

impl Default for Chessboard {
    /// Create a new chessboard set in the starting position.
    fn default() -> Self {
        let mut pieces = [[Bitboard::new(0); PieceType::COUNT]; Side::COUNT];

        let w = Side::White as usize;
        let b = Side::Black as usize;

        // Assign white pieces
        pieces[w][PieceType::Pawn as usize] = Bitboard::new(0x0000_0000_0000_FF00);
        pieces[w][PieceType::Rook as usize] = Bitboard::new(0x0000_0000_0000_0081);
        pieces[w][PieceType::Knight as usize] = Bitboard::new(0x0000_0000_0000_0042);
        pieces[w][PieceType::Bishop as usize] = Bitboard::new(0x0000_0000_0000_0024);
        pieces[w][PieceType::Queen as usize] = Bitboard::new(0x0000_0000_0000_0008);
        pieces[w][PieceType::King as usize] = Bitboard::new(0x0000_0000_0000_0010);

        // Assign black pieces
        pieces[b][PieceType::Pawn as usize] = Bitboard::new(0x00FF_0000_0000_0000);
        pieces[b][PieceType::Rook as usize] = Bitboard::new(0x8100_0000_0000_0000);
        pieces[b][PieceType::Knight as usize] = Bitboard::new(0x4200_0000_0000_0000);
        pieces[b][PieceType::Bishop as usize] = Bitboard::new(0x2400_0000_0000_0000);
        pieces[b][PieceType::Queen as usize] = Bitboard::new(0x0800_0000_0000_0000);
        pieces[b][PieceType::King as usize] = Bitboard::new(0x1000_0000_0000_0000);

        Self { pieces }
    }
}

impl Display for Chessboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let board = self.occupied();

        // Use write! instead of writeln! since a Bitboard as a string will
        // have an extra /n anyways
        write!(f, "{}", board)?;

        Ok(())
    }
}

/// A bitboard representation of a chessboard
#[derive(Default, Clone, Copy)]
struct Bitboard {
    bitboard: u64,
}

impl Bitboard {
    fn new(bitboard: u64) -> Self {
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

impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.bitboard |= rhs.bitboard
    }
}

impl BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Bitboard::new(self.bitboard | rhs.bitboard)
    }
}

#[derive(EnumCount)]
enum Side {
    White = 0,
    Black = 1,
}

#[derive(EnumCount)]
enum PieceType {
    King = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    Pawn = 5,
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    mod chessboard {
        use super::*;

        #[test]
        fn default_board() {
            let chessboard = Chessboard::default();
            let expected = indoc! {r#"
                11111111
                11111111
                00000000
                00000000
                00000000
                00000000
                11111111
                11111111
            "#};

            assert_eq!(chessboard.to_string(), expected);
        }
    }

    mod bitboard {
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
}
