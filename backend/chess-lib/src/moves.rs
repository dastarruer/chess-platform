use strum::{EnumCount, EnumIter, IntoEnumIterator};

use crate::{Bitboard, Piece, PieceType, Side, Square, square::Offset};

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
        let knight_moves = Self::precalculate_piece_moves::<KnightJump>();
        let king_moves = Self::precalculate_piece_moves::<KingMove>();

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

        for piece_kind in PieceType::iter() {
            let piece = Piece {
                kind: piece_kind,
                side: active_color,
            };

            let mut bitboard = pieces[active_color as usize][piece_kind as usize];
            let mut possible_squares = Vec::new();

            while let Some(square) = bitboard.pop() {
                possible_squares.push(square);
            }

            for square in possible_squares {
                match piece_kind {
                    PieceType::Knight => {
                        let mut moves = self.knight_moves[square as usize];
                        while let Some(move_square) = moves.pop() {
                            if let Some(other_piece) = squares[move_square as usize]
                                && other_piece.side == active_color
                            {
                                continue;
                            }

                            legal_moves.push(Move::new(piece, square, move_square));
                        }
                    }
                    PieceType::King => {
                        // TODO: Disallow taking protected pieces
                        let mut moves = self.king_moves[square as usize];
                        while let Some(move_square) = moves.pop() {
                            if let Some(other_piece) = squares[move_square as usize]
                                && other_piece.side == active_color
                            {
                                continue;
                            }

                            legal_moves.push(Move::new(piece, square, move_square));
                        }
                    }
                    PieceType::Rook => {
                        legal_moves.append(&mut RookMove::generate_legal_moves(
                            &square, piece, friendly, opposition,
                        ));
                    }
                    PieceType::Bishop => {
                        legal_moves.append(&mut BishopMove::generate_legal_moves(
                            &square, piece, friendly, opposition,
                        ));
                    }
                    PieceType::Queen => {
                        legal_moves.append(&mut QueenMove::generate_legal_moves(
                            &square, piece, friendly, opposition,
                        ));
                    }
                    _ => {}
                }
            }
        }

        legal_moves
    }

    fn precalculate_piece_moves<T>() -> [Bitboard; Square::COUNT]
    where
        T: IntoEnumIterator + Copy,
        Offset: From<T>,
    {
        let mut piece_moves = [Bitboard::empty(); Square::COUNT];

        for from in Square::iter() {
            let mut moves = Bitboard::empty();

            for mv in T::iter() {
                let Ok(to) = from.try_offset(Offset::from(mv)) else {
                    continue;
                };
                let jump_mask = Bitboard::new(to.mask());

                moves |= jump_mask;
            }

            piece_moves[from as usize] = moves;
        }

        piece_moves
    }
}

trait SlidingPieceMove
where
    Self: IntoEnumIterator + Copy,
    Offset: std::convert::From<Self>,
{
    const PIECE: PieceType;

    fn generate_legal_moves(
        from: &Square,
        piece: Piece,
        friendly: Bitboard,
        opposition: Bitboard,
    ) -> Vec<Move>
    where
        Offset: std::convert::From<Self>,
    {
        let mut legal_moves = Vec::new();

        for mv in Self::iter() {
            let mut to = *from;
            while let Ok(next) = to.try_offset(Offset::from(mv)) {
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
}

#[derive(Debug, Clone, Copy, EnumIter)]
enum RookMove {
    North,
    South,
    East,
    West,
}

impl From<RookMove> for Offset {
    fn from(value: RookMove) -> Self {
        match value {
            RookMove::North => Self::NORTH,
            RookMove::South => Self::SOUTH,
            RookMove::East => Self::EAST,
            RookMove::West => Self::WEST,
        }
    }
}

impl SlidingPieceMove for RookMove {
    const PIECE: PieceType = PieceType::Rook;
}

#[derive(Debug, Clone, Copy, EnumIter)]
enum BishopMove {
    NorthWest,
    SouthEast,
    NorthEast,
    SouthWest,
}

impl From<BishopMove> for Offset {
    fn from(value: BishopMove) -> Self {
        match value {
            BishopMove::NorthWest => Self::NORTH_WEST,
            BishopMove::SouthEast => Self::SOUTH_EAST,
            BishopMove::NorthEast => Self::NORTH_EAST,
            BishopMove::SouthWest => Self::SOUTH_WEST,
        }
    }
}

impl SlidingPieceMove for BishopMove {
    const PIECE: PieceType = PieceType::Bishop;
}

#[derive(Debug, Clone, Copy, EnumIter)]
enum QueenMove {
    North,
    South,
    East,
    West,
    NorthWest,
    SouthEast,
    NorthEast,
    SouthWest,
}

impl From<QueenMove> for Offset {
    fn from(value: QueenMove) -> Self {
        match value {
            QueenMove::North => Self::NORTH,
            QueenMove::South => Self::SOUTH,
            QueenMove::East => Self::EAST,
            QueenMove::West => Self::WEST,
            QueenMove::NorthWest => Self::NORTH_WEST,
            QueenMove::SouthEast => Self::SOUTH_EAST,
            QueenMove::NorthEast => Self::NORTH_EAST,
            QueenMove::SouthWest => Self::SOUTH_WEST,
        }
    }
}

impl SlidingPieceMove for QueenMove {
    const PIECE: PieceType = PieceType::Queen;
}

#[derive(Debug, Clone, Copy, EnumIter)]
enum KingMove {
    North,
    South,
    East,
    West,
    NorthWest,
    SouthEast,
    NorthEast,
    SouthWest,
}

impl From<KingMove> for Offset {
    fn from(value: KingMove) -> Self {
        match value {
            KingMove::North => Self::NORTH,
            KingMove::South => Self::SOUTH,
            KingMove::East => Self::EAST,
            KingMove::West => Self::WEST,
            KingMove::NorthWest => Self::NORTH_WEST,
            KingMove::SouthEast => Self::SOUTH_EAST,
            KingMove::NorthEast => Self::NORTH_EAST,
            KingMove::SouthWest => Self::SOUTH_WEST,
        }
    }
}

#[derive(Debug, Clone, Copy, EnumIter)]
#[allow(clippy::enum_variant_names)]
/// Store bit shift offset values for each possible knight jump.
enum KnightJump {
    TwoUpOneLeft,
    TwoUpOneRight,
    TwoRightOneUp,
    TwoRightOneDown,
    TwoDownOneLeft,
    TwoDownOneRight,
    TwoLeftOneUp,
    TwoLeftOneDown,
}

impl From<KnightJump> for Offset {
    fn from(value: KnightJump) -> Self {
        match value {
            KnightJump::TwoUpOneLeft => Offset::TWO_UP_ONE_LEFT,
            KnightJump::TwoUpOneRight => Offset::TWO_UP_ONE_RIGHT,
            KnightJump::TwoRightOneUp => Offset::TWO_RIGHT_ONE_UP,
            KnightJump::TwoRightOneDown => Offset::TWO_RIGHT_ONE_DOWN,
            KnightJump::TwoDownOneLeft => Offset::TWO_DOWN_ONE_LEFT,
            KnightJump::TwoDownOneRight => Offset::TWO_DOWN_ONE_RIGHT,
            KnightJump::TwoLeftOneUp => Offset::TWO_LEFT_ONE_UP,
            KnightJump::TwoLeftOneDown => Offset::TWO_LEFT_ONE_DOWN,
        }
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
