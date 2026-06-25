use anyhow::{Context, anyhow};
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

    /// Returns the [`Rank`] of the [`Square`].
    pub fn rank(&self) -> Rank {
        (*self as u8 / Rank::COUNT as u8)
            .try_into()
            .expect("Converting square index to rank should not be invalid")
    }

    /// Returns the [`File`] of the [`Square`].
    pub fn file(&self) -> File {
        (*self as u8 % File::COUNT as u8)
            .try_into()
            .expect("Converting square index to rank should not be invalid")
    }
}

impl TryFrom<&str> for Square {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() != 2 {
            return Err(anyhow!("'{value}' is an invalid square"));
        }

        let mut chars = value.chars();
        let file = File::try_from(
            chars
                .next()
                .expect("Square should have at least two characters"),
        )?;
        let rank = Rank::try_from(
            chars
                .next()
                .expect("Square should have at least two characters"),
        )?;

        Ok(Square::from_coordinates(file, rank))
    }
}

/// Provides a mechanism to advance an enum variant forward by a specific
/// offset.
///
/// This trait is meant to be used for enum variants that can go out of bounds,
/// including chess files and ranks.
pub trait TryNext: Into<u8> + TryFrom<u8> + EnumCount + std::fmt::Display + Copy
where
    anyhow::Error: From<<Self as TryFrom<u8>>::Error> + Send + Sync,
{
    /// Advance the current value forward `n` steps.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - Resulting conversion goes out of bounds.
    ///
    /// /// # Examples
    ///
    /// ```rust
    /// # use chess_lib::square::{Rank, TryNext};
    /// let rank = Rank::R7;
    /// assert_eq!(rank.next(1).unwrap(), Rank::R8);
    /// assert!(rank.next(2).is_err()); // Would result in out-of-bounds rank
    /// ```
    fn next(self, n: u8) -> anyhow::Result<Self> {
        let next = self
            .into()
            .checked_add(n)
            .with_context(|| format!("Adding {n} to {self} is invalid as it causes an overflow"))?;

        if next > Self::COUNT as u8 {
            return Err(anyhow!(
                "Adding {n} to {self} is invalid as it goes out of bounds"
            ));
        }

        Ok(Self::try_from(next)?)
    }
}

/// Provides a mechanism to move an enum variant backwards by a specific
/// offset.
///
/// This trait is meant to be used for enum variants that can go out of bounds,
/// including chess files and ranks.
pub trait TryPrevious: Into<u8> + TryFrom<u8> + EnumCount + std::fmt::Display + Copy
where
    anyhow::Error: From<<Self as TryFrom<u8>>::Error> + Send + Sync,
{
    /// Move the current value backwards `n` steps.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - Resulting conversion goes out of bounds.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use chess_lib::square::{Rank, TryPrevious};
    /// let rank = Rank::R2;
    /// assert_eq!(rank.prev(1).unwrap(), Rank::R1);
    /// assert!(rank.prev(2).is_err()); // Would result in out-of-bounds rank
    /// ```
    fn prev(self, n: u8) -> anyhow::Result<Self> {
        let prev = self.into().checked_sub(n).with_context(|| {
            format!("Subtracting {n} from {self} is invalid as it causes an underflow")
        })?;

        Ok(Self::try_from(prev)?)
    }
}

#[derive(EnumCount, EnumIter, Clone, Copy, PartialEq, Eq, Debug, FromRepr, strum::Display)]
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

impl TryFrom<u8> for File {
    type Error = anyhow::Error;
    /// Convert a file index into a [`File`].
    ///
    /// For instance, a `value` of `0` corresponds with [`File::A`], `1`
    /// corresponds with [`File::B`], and so on.
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        File::from_repr(value).with_context(|| format!("Unable to convert {value} to File"))
    }
}

impl From<File> for u8 {
    fn from(val: File) -> Self {
        val as u8
    }
}

impl TryFrom<char> for File {
    type Error = anyhow::Error;

    /// Convert an ASCII character to a [`File`], irrespective of case.
    ///
    /// For instance, a `value` of `a` corresponds with [`File::A`], `B`
    /// corresponds with [`File::B`], and so on.
    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value.to_ascii_lowercase() {
            'a' => Ok(Self::A),
            'b' => Ok(Self::B),
            'c' => Ok(Self::C),
            'd' => Ok(Self::D),
            'e' => Ok(Self::E),
            'f' => Ok(Self::F),
            'g' => Ok(Self::G),
            'h' => Ok(Self::H),
            _ => Err(anyhow!("'{value}' is not a valid file")),
        }
    }
}

impl TryNext for File {}
impl TryPrevious for File {}

#[derive(EnumCount, EnumIter, Clone, Copy, PartialEq, Eq, Debug, FromRepr, strum::Display)]
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

impl TryFrom<u8> for Rank {
    type Error = anyhow::Error;

    /// Convert a rank index into a [`Rank`].
    ///
    /// For instance, a `value` of `0` corresponds with [`Rank::R1`], `1`
    /// corresponds with [`Rank::R2`], and so on.
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Rank::from_repr(value).with_context(|| format!("Unable to convert {value} to Rank"))
    }
}

impl TryFrom<char> for Rank {
    type Error = anyhow::Error;

    /// Convert an ASCII character to a [`Rank`].
    ///
    /// For instance, a `value` of `1` corresponds with [`Rank::R1`], `2`
    /// corresponds with [`Rank::R2`], and so on.
    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value.to_ascii_lowercase() {
            '1' => Ok(Self::R1),
            '2' => Ok(Self::R2),
            '3' => Ok(Self::R3),
            '4' => Ok(Self::R4),
            '5' => Ok(Self::R5),
            '6' => Ok(Self::R6),
            '7' => Ok(Self::R7),
            '8' => Ok(Self::R8),
            _ => Err(anyhow!("'{value}' is not a valid rank")),
        }
    }
}

impl From<Rank> for u8 {
    fn from(val: Rank) -> Self {
        val as u8
    }
}

impl TryNext for Rank {}
impl TryPrevious for Rank {}

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

    #[test]
    fn get_rank_from_square() {
        assert_eq!(Square::A1.rank(), Rank::R1);
        assert_eq!(Square::H1.rank(), Rank::R1);
        assert_eq!(Square::A2.rank(), Rank::R2);
        assert_eq!(Square::E4.rank(), Rank::R4);
        assert_eq!(Square::H8.rank(), Rank::R8);
    }

    #[test]
    fn get_file_from_square() {
        assert_eq!(Square::A1.file(), File::A);
        assert_eq!(Square::A8.file(), File::A);
        assert_eq!(Square::B1.file(), File::B);
        assert_eq!(Square::E4.file(), File::E);
        assert_eq!(Square::H8.file(), File::H);
    }
}
