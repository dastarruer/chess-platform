use strum::{EnumCount, EnumIter, IntoEnumIterator};

use crate::{Bitboard, Offset, Piece, PieceType, Side, Square, square::Rank};

/// All pieces whose moves can be pregenerated upon initialization, rather than
/// needing to be generated dynamically based on the position. These pieces are
/// non-sliding, meaning they don't 'slide' across the board.
///
/// Technically, pawns are also non-sliding, but since pawns cannot be
/// pregenerated upon initialization, they aren't present here.
#[derive(EnumIter, Clone, Copy)]
enum NonSlidingPieceType {
    King,
    Knight,
}

impl From<NonSlidingPieceType> for PieceType {
    fn from(value: NonSlidingPieceType) -> Self {
        match value {
            NonSlidingPieceType::King => Self::King,
            NonSlidingPieceType::Knight => Self::Knight,
        }
    }
}

/// All pieces whose moves must be generated dynamically for every position.
/// These pieces are sliding, meaning they can 'slide' around the board.
#[derive(EnumIter, Clone, Copy)]
enum SlidingPieceType {
    Rook,
    Bishop,
    Queen,
}

impl From<SlidingPieceType> for PieceType {
    fn from(value: SlidingPieceType) -> Self {
        match value {
            SlidingPieceType::Bishop => PieceType::Bishop,
            SlidingPieceType::Queen => PieceType::Queen,
            SlidingPieceType::Rook => PieceType::Rook,
        }
    }
}

#[derive(PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct Move {
    pub piece: Piece,
    pub from: Square,
    pub to: Square,
}

impl Move {
    pub fn new(piece: Piece, from: Square, to: Square) -> Self {
        Self { piece, from, to }
    }
}

pub(super) struct MoveGenerator {
    pub(super) knight_moves: [Bitboard; Square::COUNT],
    king_moves: [Bitboard; Square::COUNT],
}

impl MoveGenerator {
    pub(super) fn new() -> Self {
        let knight_moves = Self::precalculate_piece_moves(NonSlidingPieceType::Knight);
        let king_moves = Self::precalculate_piece_moves(NonSlidingPieceType::King);

        MoveGenerator {
            knight_moves,
            king_moves,
        }
    }

    pub(super) fn legal_moves(
        &self,
        active_color: Side,
        pieces: &[[Bitboard; PieceType::COUNT]; Side::COUNT],
        squares: &[Option<Piece>; Square::COUNT],
    ) -> Vec<Move> {
        let mut legal_moves = Vec::new();
        let friendly = pieces[active_color as usize]
            .iter()
            .fold(Bitboard::empty(), |friendly, bb| friendly | *bb);
        let opposition = pieces[active_color.opposite() as usize]
            .iter()
            .fold(Bitboard::empty(), |opposing, bb| opposing | *bb);

        for kind in NonSlidingPieceType::iter() {
            let piece = Piece {
                kind: PieceType::from(kind),
                side: active_color,
            };

            let table = match kind {
                NonSlidingPieceType::King => self.king_moves,
                NonSlidingPieceType::Knight => self.knight_moves,
            };

            let mut bitboard = pieces[active_color as usize][piece.kind as usize];
            while let Some(square) = bitboard.pop() {
                let mut moves = table[square as usize];
                while let Some(move_square) = moves.pop() {
                    if let Some(other_piece) = squares[move_square as usize]
                        && other_piece.side == active_color
                    {
                        continue;
                    }

                    legal_moves.push(Move::new(piece, square, move_square));
                }
            }
        }

        for kind in SlidingPieceType::iter() {
            let piece = Piece {
                kind: PieceType::from(kind),
                side: active_color,
            };

            let mut bitboard = pieces[active_color as usize][piece.kind as usize];
            while let Some(square) = bitboard.pop() {
                legal_moves.append(&mut Self::generate_legal_moves(
                    &square, piece.side, kind, friendly, opposition,
                ));
            }
        }

        // Handle pawns separately because they are special
        // just like me...
        let piece = Piece {
            kind: PieceType::Pawn,
            side: active_color,
        };
        let mut pawns = pieces[active_color as usize][piece.kind as usize];
        while let Some(square) = pawns.pop() {
            legal_moves.append(&mut Self::generate_legal_pawn_moves(
                &square, piece.side, friendly, opposition,
            ));
        }

        legal_moves
    }

    fn precalculate_piece_moves(piece: NonSlidingPieceType) -> [Bitboard; Square::COUNT] {
        let piece = PieceType::from(piece);
        let mut piece_moves = [Bitboard::empty(); Square::COUNT];

        for from in Square::iter() {
            let mut moves = Bitboard::empty();

            for offset in piece.offsets() {
                let Ok(to) = from.try_offset(offset) else {
                    continue;
                };
                let jump_mask = Bitboard::new(to.mask());

                moves |= jump_mask;
            }

            piece_moves[from as usize] = moves;
        }

        piece_moves
    }

    fn generate_legal_moves(
        from: &Square,
        side: Side,
        kind: SlidingPieceType,
        friendly: Bitboard,
        opposition: Bitboard,
    ) -> Vec<Move> {
        let piece = Piece {
            kind: PieceType::from(kind),
            side,
        };
        let mut legal_moves = Vec::new();

        for offset in piece.kind.offsets() {
            let mut to = *from;
            while let Ok(next) = to.try_offset(offset) {
                to = next;
                let is_friendly = friendly.contains(to);
                let is_opposition = opposition.contains(to);

                if is_friendly {
                    break;
                }

                legal_moves.push(Move::new(piece, *from, to));

                // Since an opposite piece can be taken, it's still a valid move
                if is_opposition {
                    break;
                }
            }
        }

        legal_moves
    }

    fn generate_legal_pawn_moves(
        from: &Square,
        side: Side,
        friendly: Bitboard,
        opposition: Bitboard,
    ) -> Vec<Move> {
        let piece: Piece = Piece {
            kind: PieceType::Pawn,
            side,
        };

        let is_first_move = (from.rank() == Rank::R2 && side == Side::White)
            || (from.rank() == Rank::R7 && side == Side::Black);
        let mut offsets = match (is_first_move, side) {
            (true, Side::White) => [Some(Offset::NORTH), Some(Offset::TWO_UP)],
            (true, Side::Black) => [Some(Offset::SOUTH), Some(Offset::TWO_DOWN)],
            (false, Side::White) => [Some(Offset::NORTH), None],
            (false, Side::Black) => [Some(Offset::SOUTH), None],
        }
        .into_iter();

        let mut legal_moves = Vec::new();
        while let Some(Some(offset)) = offsets.next()
            && let Ok(to) = from.try_offset(&offset)
        {
            let is_occupied = friendly.contains(to) || opposition.contains(to);

            // Since the offsets start from one-square moves, we can break the loop as soon as a move is not possible (im a genius)
            if is_occupied {
                break;
            }

            legal_moves.push(Move::new(piece, *from, to));
        }

        legal_moves
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn knight_moves() {
        let generator = MoveGenerator::new();

        // --- Case 1: Center Square (E4) ---
        // A knight on E4 should have exactly 8 valid moves:
        // D6, F6, C5, G5, C3, G3, D2, F2
        let e4_index = Square::E4 as usize;
        let e4_attacks = generator.knight_moves[e4_index];

        assert_eq!(
            e4_attacks.count_ones(),
            8,
            "Knight on E4 should have exactly 8 legal targets"
        );
        assert!(e4_attacks.is_set(Square::D6));
        assert!(e4_attacks.is_set(Square::F6));
        assert!(e4_attacks.is_set(Square::G5));
        assert!(e4_attacks.is_set(Square::F2));

        // --- Case 2: Corner Square (A1) ---
        // A knight on A1 is highly restricted. It can only jump to B3 and C2.
        let a1_index = Square::A1 as usize;
        let a1_attacks = generator.knight_moves[a1_index];

        assert_eq!(
            a1_attacks.count_ones(),
            2,
            "Knight on A1 should only have 2 legal targets"
        );
        assert!(a1_attacks.is_set(Square::B3));
        assert!(a1_attacks.is_set(Square::C2));

        // Explicitly verify it didn't illegally wrap around to the H-file
        assert!(!a1_attacks.is_set(Square::H2));

        // --- Case 3: Edge Square (A4) ---
        // A knight on A4 can move to B6, C5, C3, and B2 (4 moves).
        let a4_index = Square::A4 as usize;
        let a4_attacks = generator.knight_moves[a4_index];

        assert_eq!(
            a4_attacks.count_ones(),
            4,
            "Knight on A4 should only have 4 legal targets due to left-edge filtering"
        );
        assert!(a4_attacks.is_set(Square::B6));
        assert!(a4_attacks.is_set(Square::C5));
        assert!(a4_attacks.is_set(Square::C3));
        assert!(a4_attacks.is_set(Square::B2));
    }

    #[test]
    fn king_moves() {
        let generator = MoveGenerator::new();

        // --- Case 1: Center Square (E4) ---
        // A King on E4 should access all 8 surrounding squares:
        // D5, E5, F5, D4, F4, D3, E3, F3
        let e4_attacks = generator.king_moves[Square::E4 as usize];
        assert_eq!(e4_attacks.count_ones(), 8, "King on E4 should have 8 moves");
        assert!(e4_attacks.is_set(Square::D5));
        assert!(e4_attacks.is_set(Square::E5));
        assert!(e4_attacks.is_set(Square::F3));

        // --- Case 2: Corner Square (A1) ---
        // Restricted to 3 moves: A2, B2, B1.
        // West/Southwards moves should be perfectly blocked by boundaries.
        let a1_attacks = generator.king_moves[Square::A1 as usize];
        assert_eq!(
            a1_attacks.count_ones(),
            3,
            "King on A1 should only have 3 moves"
        );
        assert!(a1_attacks.is_set(Square::A2));
        assert!(a1_attacks.is_set(Square::B2));
        assert!(a1_attacks.is_set(Square::B1));

        // Assert it didn't wrap horizontally to the H-file
        assert!(!a1_attacks.is_set(Square::H1));
        assert!(!a1_attacks.is_set(Square::H2));

        // --- Case 3: Edge Square (H4) ---
        // Restricted to 5 moves: H5, G5, G4, G3, H3.
        // Eastward moves must be caught by the A_FILE mask.
        let h4_attacks = generator.king_moves[Square::H4 as usize];
        assert_eq!(
            h4_attacks.count_ones(),
            5,
            "King on H4 should only have 5 moves"
        );
        assert!(h4_attacks.is_set(Square::H5));
        assert!(h4_attacks.is_set(Square::G4));

        // Assert it didn't wrap to the A-file
        assert!(!h4_attacks.is_set(Square::A4));
        assert!(!h4_attacks.is_set(Square::A5));
    }
}
