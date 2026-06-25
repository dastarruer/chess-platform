#![allow(dead_code)]

pub mod fen;
mod moves;
pub mod square;

use std::{
    fmt::Display,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign},
};

use strum::{EnumCount, EnumIter, IntoEnumIterator};

use crate::{fen::FENString, square::Square};

pub struct Chessboard {
    pieces: [[Bitboard; PieceType::COUNT]; Side::COUNT],
    game_stats: GameStats,
}

impl Chessboard {
    /// Create a new [`Chessboard`] from a FEN string.
    ///
    /// See [`FENString`] for details.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - `fen_str` is an invalid FEN string.
    fn new(fen_str: &str) -> anyhow::Result<Self> {
        let fen_str = FENString::try_parse(fen_str)?;
        let mut pieces = [[Bitboard::empty(); PieceType::COUNT]; Side::COUNT];

        for side in Side::iter() {
            for piece in PieceType::iter() {
                for square in fen_str.pieces(side, piece) {
                    pieces[side as usize][piece as usize] |= Bitboard::new(square.mask());
                }
            }
        }

        Ok(Chessboard {
            pieces,
            game_stats: fen_str.game_stats,
        })
    }

    /// Return a `Bitboard` containing squares occupied by a specific piece
    /// type.
    fn occupied_piece(&self, piece: PieceType) -> Bitboard {
        let mut board = Bitboard::empty();

        for side_idx in 0..Side::COUNT {
            let piece_board = self.pieces[side_idx][piece as usize];
            board |= piece_board;
        }

        board
    }

    /// Return a `Bitboard` containing squares occupied by all pieces.
    fn occupied(&self) -> Bitboard {
        let mut board = Bitboard::empty();

        for piece_board in self.pieces.as_flattened() {
            board |= *piece_board;
        }

        board
    }
}

impl Default for Chessboard {
    /// Create a new chessboard in the starting position.
    fn default() -> Self {
        const STARTING_FEN_STR: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        Chessboard::new(STARTING_FEN_STR)
            .expect("Parsing starting position should not throw an error")
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

/// A bitboard representation of a chessboard.
#[derive(Default, Clone, Copy)]
struct Bitboard {
    bitboard: u64,
}

impl Bitboard {
    fn new(bitboard: u64) -> Self {
        Self { bitboard }
    }

    fn empty() -> Self {
        Self { bitboard: 0 }
    }

    #[cfg(test)]
    fn is_set(&self, square: Square) -> bool {
        self.bitboard & square.mask() != 0
    }

    #[cfg(test)]
    fn count_ones(&self) -> u32 {
        self.bitboard.count_ones()
    }
}

impl Display for Bitboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Chunk each square into individual ranks (rows in chess lingo)
        // Reverse so 1st file (colums in chess lingo) is printed first
        for rank in Square::iter().collect::<Vec<Square>>().chunks(8).rev() {
            for square in rank {
                // If the bit is not equal to 1, the bit must be 0
                let bit = ((self.bitboard & square.mask()) != 0) as u8;

                write!(f, "{}", bit)?;
            }

            // Every 8 bits, add a newline
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

impl BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Bitboard::new(self.bitboard & rhs.bitboard)
    }
}

impl BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.bitboard &= rhs.bitboard
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GameStats {
    active_color: Side,
    castle_rights: [CastleRights; Side::COUNT],
    en_passant_target: Option<Square>,

    /// Number of moves made in a row by each side without any pawn advances or
    /// piece captures.
    ///
    /// For instance, if white makes one move, and black makes another, each
    /// made without capturing a piece or moving a pawn, two halfmoves have
    /// been made. This is used to enforce the 50-move rule, which ends the
    /// game in a draw after 100 halfmoves.
    halfmoves: u8,

    /// Number of completed turns in the game.
    ///
    /// For instance, if white makes one move, and black makes another, one
    /// fullmove has been made.
    fullmoves: u16,
}

#[derive(EnumCount, EnumIter, Debug, Eq, PartialEq, Clone, Copy)]
pub enum Side {
    White = 0,
    Black = 1,
}

#[derive(EnumCount, EnumIter, Clone, Copy)]
pub enum PieceType {
    King = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    Pawn = 5,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum CastleRights {
    King,
    Queen,
    KingQueen,
    Neither,
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

        #[test]
        fn default_knight_board() {
            let chessboard = Chessboard::default();
            let expected = indoc! {r#"
                01000010
                00000000
                00000000
                00000000
                00000000
                00000000
                00000000
                01000010
            "#};

            assert_eq!(
                chessboard.occupied_piece(PieceType::Knight).to_string(),
                expected
            );
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

        #[test]
        fn board_corners_no_flip() {
            let a8_mask = Square::A8.mask();
            let h1_mask = Square::H1.mask();

            let board = Bitboard::new(a8_mask | h1_mask);

            let expected = indoc! {r#"
                10000000
                00000000
                00000000
                00000000
                00000000
                00000000
                00000000
                00000001
            "#};

            assert_eq!(board.to_string(), expected);
        }
    }
}
