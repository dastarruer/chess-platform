#![allow(dead_code)]

pub mod fen;
mod moves;
pub mod square;

use std::{
    fmt::Display,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign},
};

use strum::{EnumCount, EnumIter, IntoEnumIterator};

use crate::{
    fen::FENString,
    moves::{Move, MoveGenerator},
    square::Square,
};

pub struct Chessboard {
    pieces: [[Bitboard; PieceType::COUNT]; Side::COUNT],
    /// Stores information on which squares contain which piece.
    ///
    /// Bitboards are inefficient when it comes to figuring out what piece is
    /// on which square, so this is done with an array.
    squares: [Option<Piece>; Square::COUNT],
    move_generator: MoveGenerator,
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
        let move_generator = MoveGenerator::new();
        let fen_str = FENString::try_parse(fen_str)?;
        let mut pieces = [[Bitboard::empty(); PieceType::COUNT]; Side::COUNT];
        let mut squares = [None; Square::COUNT];

        for side in Side::iter() {
            for piece in PieceType::iter() {
                for square in fen_str.pieces(side, piece) {
                    pieces[side as usize][piece as usize] |= Bitboard::new(square.mask());

                    let piece = Piece { kind: piece, side };
                    squares[*square as usize] = Some(piece);
                }
            }
        }

        Ok(Chessboard {
            pieces,
            squares,
            move_generator,
            game_stats: fen_str.game_stats,
        })
    }

    pub fn legal_moves(&self) -> Vec<Move> {
        let active_color = self.game_stats.active_color;
        self.move_generator
            .legal_moves(active_color, &self.pieces, &self.squares)
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

    /// Return a `Bitboard` containing squares occupied by a specific side.
    fn occupied_side(&self, side: Side) -> Bitboard {
        let mut board = Bitboard::empty();

        for piece in PieceType::iter() {
            let piece_board = self.pieces[side as usize][piece as usize];
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Piece {
    pub kind: PieceType,
    pub side: Side,
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

    fn is_empty(&self) -> bool {
        self.bitboard == 0
    }

    /// Pops the least significant bit from the bitboard, and returns it as a
    /// [`Square`].
    ///
    /// Returns `None` if bitboard is empty.
    fn pop(&mut self) -> Option<Square> {
        // Check early to prevent overflow
        if self.bitboard == 0 {
            return None;
        }

        let index = self.bitboard.trailing_zeros() as u8;
        self.bitboard &= self.bitboard - 1; // Fun trick to quickly remove the lsb

        Square::try_from(index).ok()
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

#[derive(EnumCount, EnumIter, Clone, Copy, Debug, PartialEq, Eq)]
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

            let expected_bitboard = indoc! {r#"
                    11111111
                    11111111
                    00000000
                    00000000
                    00000000
                    00000000
                    11111111
                    11111111
                "#};

            assert_eq!(chessboard.to_string(), expected_bitboard);

            use PieceType::*;
            use Side::*;

            let mut expected = [None; Square::COUNT];

            // White
            expected[Square::A1 as usize] = Some(Piece {
                kind: Rook,
                side: White,
            });
            expected[Square::B1 as usize] = Some(Piece {
                kind: Knight,
                side: White,
            });
            expected[Square::C1 as usize] = Some(Piece {
                kind: Bishop,
                side: White,
            });
            expected[Square::D1 as usize] = Some(Piece {
                kind: Queen,
                side: White,
            });
            expected[Square::E1 as usize] = Some(Piece {
                kind: King,
                side: White,
            });
            expected[Square::F1 as usize] = Some(Piece {
                kind: Bishop,
                side: White,
            });
            expected[Square::G1 as usize] = Some(Piece {
                kind: Knight,
                side: White,
            });
            expected[Square::H1 as usize] = Some(Piece {
                kind: Rook,
                side: White,
            });

            for file in [
                Square::A2,
                Square::B2,
                Square::C2,
                Square::D2,
                Square::E2,
                Square::F2,
                Square::G2,
                Square::H2,
            ] {
                expected[file as usize] = Some(Piece {
                    kind: Pawn,
                    side: White,
                });
            }

            // Black
            expected[Square::A8 as usize] = Some(Piece {
                kind: Rook,
                side: Black,
            });
            expected[Square::B8 as usize] = Some(Piece {
                kind: Knight,
                side: Black,
            });
            expected[Square::C8 as usize] = Some(Piece {
                kind: Bishop,
                side: Black,
            });
            expected[Square::D8 as usize] = Some(Piece {
                kind: Queen,
                side: Black,
            });
            expected[Square::E8 as usize] = Some(Piece {
                kind: King,
                side: Black,
            });
            expected[Square::F8 as usize] = Some(Piece {
                kind: Bishop,
                side: Black,
            });
            expected[Square::G8 as usize] = Some(Piece {
                kind: Knight,
                side: Black,
            });
            expected[Square::H8 as usize] = Some(Piece {
                kind: Rook,
                side: Black,
            });

            for file in [
                Square::A7,
                Square::B7,
                Square::C7,
                Square::D7,
                Square::E7,
                Square::F7,
                Square::G7,
                Square::H7,
            ] {
                expected[file as usize] = Some(Piece {
                    kind: Pawn,
                    side: Black,
                });
            }

            assert_eq!(chessboard.squares, expected);
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

        #[test]
        fn pop() {
            let h2 = Square::H2;
            let mut board = Bitboard::new(h2.mask());
            assert_eq!(board.pop().expect("Bitboard should not be empty"), h2);
        }
    }

    mod legal_moves {
        use super::*;
        use crate::Chessboard;

        #[test]
        fn king_moves() {
            const KING: Piece = Piece {
                kind: PieceType::King,
                side: Side::White,
            };

            // King moves in starting position
            let chessboard = Chessboard::default();
            let moves = chessboard.legal_moves();
            for mv in moves {
                assert_ne!(mv.piece.kind, PieceType::King);
            }

            // King moves with capturable enemy piece and uncapturable
            // same-side piece
            const FROM: Square = Square::D4;
            let chessboard = Chessboard::new("8/8/8/8/2PKp3/8/8/8 w - - 0 1")
                .expect("FEN string should be valid");
            let moves = chessboard.legal_moves();
            let expected = vec![
                Move::new(KING, FROM, Square::E4),
                Move::new(KING, FROM, Square::E5),
                Move::new(KING, FROM, Square::E3),
                Move::new(KING, FROM, Square::D3),
                Move::new(KING, FROM, Square::D5),
                Move::new(KING, FROM, Square::C5),
                Move::new(KING, FROM, Square::C3),
            ];
            assert_eq!(moves.len(), expected.len());
            for mv in expected {
                assert!(moves.contains(&mv));
            }
        }

        #[test]
        fn knight_moves() {
            // Knight moves in starting position
            let chessboard = Chessboard::default();
            let moves = chessboard.legal_moves();
            let expected = vec![
                Move::new(
                    Piece {
                        kind: PieceType::Knight,
                        side: Side::White,
                    },
                    Square::B1,
                    Square::A3,
                ),
                Move::new(
                    Piece {
                        kind: PieceType::Knight,
                        side: Side::White,
                    },
                    Square::B1,
                    Square::C3,
                ),
                Move::new(
                    Piece {
                        kind: PieceType::Knight,
                        side: Side::White,
                    },
                    Square::G1,
                    Square::F3,
                ),
                Move::new(
                    Piece {
                        kind: PieceType::Knight,
                        side: Side::White,
                    },
                    Square::G1,
                    Square::H3,
                ),
            ];
            assert_eq!(moves.len(), expected.len());
            for mv in expected {
                assert!(moves.contains(&mv));
            }
        }
    }
}
