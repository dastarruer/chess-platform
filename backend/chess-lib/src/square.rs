use strum::{EnumCount, EnumIter};

#[derive(EnumCount, EnumIter, Clone, Copy, PartialEq, Eq, Debug)]
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
}
