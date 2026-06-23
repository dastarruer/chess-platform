use strum::{EnumCount, EnumIter, FromRepr};

#[derive(EnumCount, EnumIter, Clone, Copy, PartialEq, Eq, Debug, FromRepr)]
#[repr(u8)]
pub enum Square {
    A1 = 0,
    B1,
    C1,
    D1,
    E1,
    F1,
    G1,
    H1,
    A2,
    B2,
    C2,
    D2,
    E2,
    F2,
    G2,
    H2,
    A3,
    B3,
    C3,
    D3,
    E3,
    F3,
    G3,
    H3,
    A4,
    B4,
    C4,
    D4,
    E4,
    F4,
    G4,
    H4,
    A5,
    B5,
    C5,
    D5,
    E5,
    F5,
    G5,
    H5,
    A6,
    B6,
    C6,
    D6,
    E6,
    F6,
    G6,
    H6,
    A7,
    B7,
    C7,
    D7,
    E7,
    F7,
    G7,
    H7,
    A8,
    B8,
    C8,
    D8,
    E8,
    F8,
    G8,
    H8,
}

impl Square {
    /// Generate a bit mask where only the bit corresponding to the value of
    /// the [`Square`] is set.
    ///
    /// For example:
    ///
    /// ```rust
    /// # use chess_lib::square::Square;
    /// let square = Square::E4;
    /// let mask = square.mask();
    ///
    /// assert_eq!(mask, 1u64 << 28);
    /// ```
    ///
    /// Visually, this mask would look like this:
    ///
    /// ```plaintext
    /// 00000000
    /// 00000000
    /// 00000000
    /// 00000000
    /// 00001000
    /// 00000000
    /// 00000000
    /// 00000000
    /// ```
    ///
    /// Where E4 is the only bit set.
    pub const fn mask(self) -> u64 {
        1u64 << (self as u8)
    }

    pub fn from_coordinates(file: File, rank: Rank) -> Self {
        Self::from_repr((rank as u8 * 8) + file as u8).expect("Coordinates should not be invalid")
    }
}

#[repr(u8)]
pub enum File {
    A = 0,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

#[repr(u8)]
pub enum Rank {
    R1 = 0,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_coordinates() {
        let file = File::A;
        let rank = Rank::R1;
        let expected = Square::A1;

        assert_eq!(Square::from_coordinates(file, rank), expected);

        let file = File::C;
        let rank = Rank::R5;
        let expected = Square::C5;

        assert_eq!(Square::from_coordinates(file, rank), expected);

        let file = File::H;
        let rank = Rank::R8;
        let expected = Square::H8;

        assert_eq!(Square::from_coordinates(file, rank), expected);
    }
}
