use strum::{EnumCount, EnumIter, IntoEnumIterator};

use crate::{Bitboard, Square};

struct MoveGenerator {
    knight_moves: [Bitboard; Square::COUNT],
}

impl MoveGenerator {
    fn new() -> Self {
        let knight_moves = Self::precalculate_knight_moves();

        MoveGenerator { knight_moves }
    }

    fn precalculate_knight_moves() -> [Bitboard; Square::COUNT] {
        let mut knight_moves = [Bitboard::empty(); Square::COUNT];

        for square in Square::iter() {
            let mut moves = Bitboard::empty();
            let knight_mask = square.mask();

            for jump in KnightJump::iter() {
                let jump_mask = Bitboard::new(jump.shift(knight_mask) & jump.file_mask());

                moves |= jump_mask;
            }

            knight_moves[square as usize] = moves;
        }

        knight_moves
    }
}

#[derive(Debug, Clone, Copy, EnumIter)]
#[repr(i8)]
#[allow(clippy::enum_variant_names)]
/// Store bit shift offset values for each possible knight jump.
enum KnightJump {
    TwoUpOneLeft = 15,
    TwoUpOneRight = 17,
    TwoRightOneUp = 10,
    TwoRightOneDown = -6,
    TwoDownOneLeft = -17,
    TwoDownOneRight = -15,
    TwoLeftOneUp = 6,
    TwoLeftOneDown = -10,
}

impl KnightJump {
    /// Shift a bit mask by the offset of the jump.
    fn shift(&self, mask: u64) -> u64 {
        let shift = *self as i8;

        if shift.is_negative() {
            return mask >> shift.abs();
        }
        mask << shift
    }

    /// Generate the required mask to filter illegal wraparound moves.
    fn file_mask(&self) -> u64 {
        // Masks for files on the edge of the board
        //
        // For instance, A_FILE would look like:
        // 01111111
        // 01111111
        // 01111111
        // 01111111
        // 01111111
        // 01111111
        // 01111111
        // 01111111
        const A_FILE: u64 = 0xFEFEFEFEFEFEFEFE;
        const AB_FILE: u64 = 0xFCFCFCFCFCFCFCFC;
        const H_FILE: u64 = 0x7F7F7F7F7F7F7F7F;
        const GH_FILE: u64 = 0x3F3F3F3F3F3F3F3F;

        match self {
            // Jumps going 1 step LEFT can illegally wrap to the right edge (H-file) if starting on A.
            KnightJump::TwoUpOneLeft | KnightJump::TwoDownOneLeft => H_FILE,

            // Jumps going 2 steps LEFT can illegally wrap onto G or H if
            // starting on A or B.
            KnightJump::TwoLeftOneUp | KnightJump::TwoLeftOneDown => GH_FILE,

            // Jumps going 1 step RIGHT can illegally wrap to the left edge
            // (A-file) if starting on H.
            KnightJump::TwoUpOneRight | KnightJump::TwoDownOneRight => A_FILE,

            // Jumps going 2 steps RIGHT can illegally wrap onto A or B if
            // starting on G or H.
            KnightJump::TwoRightOneUp | KnightJump::TwoRightOneDown => AB_FILE,
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
}
