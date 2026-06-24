use anyhow::{Context, anyhow};
use strum::EnumCount;

use crate::{
    CastleRights, PieceType, Side, Square,
    square::{File, Rank, TryNext, TryPrevious},
};

/// A format to store chess positions in an easily parsable string.
///
/// See <https://www.chess.com/terms/fen_str-chess> for details.
struct FENString {
    piece_positions: [[Vec<Square>; PieceType::COUNT]; Side::COUNT],
}

impl FENString {
    fn try_parse(fen_str: &str) -> anyhow::Result<Self> {
        let mut fields = fen_str.split_ascii_whitespace();

        let position = fields.next().context("FEN string is empty")?;
        let piece_positions = Self::try_parse_position(position)?;

        let active_color = fields.next().context("FEN string is incomplete")?;
        let _active_color = Self::try_parse_active_color(active_color)?;

        let castle_rights = fields.next().context("FEN string is incomplete")?;
        let _castle_rights = Self::try_parse_castle_rights(castle_rights)?;

        Ok(Self { piece_positions })
    }

    fn try_parse_position(
        position: &str,
    ) -> anyhow::Result<[[Vec<Square>; PieceType::COUNT]; Side::COUNT]> {
        let mut piece_positions: [[Vec<Square>; PieceType::COUNT]; Side::COUNT] =
            std::array::from_fn(|_| std::array::from_fn(|_| Vec::new()));

        let mut cur_file = File::A;
        let mut cur_rank = Rank::R8; // FEN strings start from the 8th rank, not the 1st
        for char in position.chars() {
            let fen_char = FENPosChars::try_from(char)?;

            match fen_char {
                FENPosChars::NewRank => {
                    // Move to the next rank
                    cur_rank = cur_rank.prev(1)?;
                    cur_file = File::A;
                }
                FENPosChars::BlackPiece(piece) => {
                    let square = Square::from_coordinates(cur_file, cur_rank);
                    piece_positions[Side::Black as usize][piece as usize].push(square);
                    cur_file = if cur_file != File::H {
                        cur_file
                            .next(1)
                            .expect("Moving to next file should not panic")
                    } else {
                        cur_file
                    };
                }
                FENPosChars::WhitePiece(piece) => {
                    let square = Square::from_coordinates(cur_file, cur_rank);
                    piece_positions[Side::White as usize][piece as usize].push(square);
                    cur_file = if cur_file != File::H {
                        cur_file
                            .next(1)
                            .expect("Moving to next file should not panic")
                    } else {
                        cur_file
                    };
                }
                FENPosChars::EmptySquares(n) => {
                    cur_file = cur_file.next(n - 1)?;
                }
            }
        }

        Ok(piece_positions)
    }

    fn try_parse_active_color(active_color: &str) -> anyhow::Result<Side> {
        if active_color.len() != 1 {
            return Err(anyhow!(
                "FEN active color field '{active_color}' length is invalid (must be a single char)"
            ));
        }

        match active_color {
            "w" => Ok(Side::White),
            "b" => Ok(Side::Black),
            _ => Err(anyhow!(
                "FEN string contains invalid active color char '{active_color}"
            )),
        }
    }

    fn try_parse_castle_rights(
        castle_rights_field: &str,
    ) -> anyhow::Result<[CastleRights; Side::COUNT]> {
        if castle_rights_field.len() > 4 || castle_rights_field.is_empty() {
            return Err(anyhow!(
                "FEN castle rights field '{castle_rights_field}' length is invalid"
            ));
        }

        let mut castle_rights = [CastleRights::Neither; Side::COUNT];
        let mut prev_fen_char = FENCastleChars::Neither;
        for char in castle_rights_field.chars() {
            let fen_char = FENCastleChars::try_from(char)?;

            // Filter out strings like 'KkQq' which is invalid
            match (&fen_char, &prev_fen_char) {
                (FENCastleChars::King(side), FENCastleChars::King(prev_side))
                | (FENCastleChars::Queen(side), FENCastleChars::Queen(prev_side))
                    if prev_side != side =>
                {
                    return Err(anyhow!(
                        "FEN castle rights field '{castle_rights_field}' is invalid"
                    ));
                }
                _ => {}
            }

            match fen_char {
                FENCastleChars::Neither if castle_rights_field.len() == 1 => break,
                FENCastleChars::Neither => {
                    return Err(anyhow!(
                        "FEN castle rights field '{castle_rights_field}' is invalid"
                    ));
                }
                FENCastleChars::King(side) => {
                    let rights = &mut castle_rights[side as usize];
                    *rights = match *rights {
                        CastleRights::Neither => CastleRights::King,
                        _ => {
                            return Err(anyhow!(
                                "FEN castle rights field '{castle_rights_field}' is invalid"
                            ));
                        }
                    };
                }
                FENCastleChars::Queen(side) => {
                    let rights = &mut castle_rights[side as usize];
                    *rights = match *rights {
                        CastleRights::Neither => CastleRights::Queen,
                        CastleRights::King => CastleRights::KingQueen,
                        _ => {
                            return Err(anyhow!(
                                "FEN castle rights field '{castle_rights_field}' is invalid"
                            ));
                        }
                    };
                }
            }

            prev_fen_char = fen_char;
        }

        Ok(castle_rights)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum FENCastleChars {
    King(Side),
    Queen(Side),
    Neither,
}

impl TryFrom<char> for FENCastleChars {
    type Error = anyhow::Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'q' => Ok(Self::Queen(Side::Black)),
            'k' => Ok(Self::King(Side::Black)),
            'Q' => Ok(Self::Queen(Side::White)),
            'K' => Ok(Self::King(Side::White)),
            '-' => Ok(Self::Neither),
            _ => Err(anyhow!("Invalid FEN castle char '{value}'")),
        }
    }
}

enum FENPosChars {
    NewRank,
    BlackPiece(PieceType),
    WhitePiece(PieceType),
    EmptySquares(u8),
}

impl TryFrom<char> for FENPosChars {
    type Error = anyhow::Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '/' => Ok(Self::NewRank),
            'r' => Ok(Self::BlackPiece(PieceType::Rook)),
            'n' => Ok(Self::BlackPiece(PieceType::Knight)),
            'b' => Ok(Self::BlackPiece(PieceType::Bishop)),
            'q' => Ok(Self::BlackPiece(PieceType::Queen)),
            'k' => Ok(Self::BlackPiece(PieceType::King)),
            'p' => Ok(Self::BlackPiece(PieceType::Pawn)),
            'R' => Ok(Self::WhitePiece(PieceType::Rook)),
            'N' => Ok(Self::WhitePiece(PieceType::Knight)),
            'B' => Ok(Self::WhitePiece(PieceType::Bishop)),
            'Q' => Ok(Self::WhitePiece(PieceType::Queen)),
            'K' => Ok(Self::WhitePiece(PieceType::King)),
            'P' => Ok(Self::WhitePiece(PieceType::Pawn)),
            '1'..='8' => Ok(Self::EmptySquares(
                value
                    .to_digit(10)
                    .expect("EmptySquare char should be parsable")
                    .try_into()
                    .expect("u32 value should be convertable to u8"),
            )),
            _ => Err(anyhow!("Invalid FEN char '{value}'")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_starting_pos() {
        // Represents the starting position of chess
        let fen_str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let fen_str = FENString::try_parse(fen_str).expect("Starting FEN string should be valid");

        let white_idx = Side::White as usize;
        let black_idx = Side::Black as usize;
        let pawn_idx = PieceType::Pawn as usize;
        let rook_idx = PieceType::Rook as usize;

        // --- Verify White Pieces ---
        // White should have 8 pawns on the 2nd rank
        assert_eq!(fen_str.piece_positions[white_idx][pawn_idx].len(), 8);
        assert!(fen_str.piece_positions[white_idx][pawn_idx].contains(&Square::A2));
        assert!(fen_str.piece_positions[white_idx][pawn_idx].contains(&Square::H2));

        // White should have rooks on A1 and H1
        assert_eq!(fen_str.piece_positions[white_idx][rook_idx].len(), 2);
        assert!(fen_str.piece_positions[white_idx][rook_idx].contains(&Square::A1));
        assert!(fen_str.piece_positions[white_idx][rook_idx].contains(&Square::H1));

        // --- Verify Black Pieces ---
        // Black should have 8 pawns on the 7th rank
        assert_eq!(fen_str.piece_positions[black_idx][pawn_idx].len(), 8);
        assert!(fen_str.piece_positions[black_idx][pawn_idx].contains(&Square::A7));
        assert!(fen_str.piece_positions[black_idx][pawn_idx].contains(&Square::H7));

        // Black should have rooks on A8 and H8
        assert_eq!(fen_str.piece_positions[black_idx][rook_idx].len(), 2);
        assert!(fen_str.piece_positions[black_idx][rook_idx].contains(&Square::A8));
        assert!(fen_str.piece_positions[black_idx][rook_idx].contains(&Square::H8));

        // --- Sanity Checks ---
        // Make sure cross-contamination didn't happen (e.g., White pawns on Black's side)
        assert!(!fen_str.piece_positions[white_idx][pawn_idx].contains(&Square::A7));
        assert!(!fen_str.piece_positions[black_idx][pawn_idx].contains(&Square::A2));
    }

    #[test]
    fn active_color() {
        let active_color = "w";
        let expected = Side::White;
        assert_eq!(
            FENString::try_parse_active_color(active_color)
                .expect("Parsing 'w' active color should not throw an error"),
            expected
        );

        let active_color = "b";
        let expected = Side::Black;
        assert_eq!(
            FENString::try_parse_active_color(active_color)
                .expect("Parsing 'b' active color should not throw an error"),
            expected
        );

        let active_color = "B";
        assert!(FENString::try_parse_active_color(active_color).is_err());

        let active_color = "bw";
        assert!(FENString::try_parse_active_color(active_color).is_err());
    }

    #[test]
    fn castle_rights() {
        let castle_rights_field = "KQkq";
        let expected = [CastleRights::KingQueen, CastleRights::KingQueen];
        assert_eq!(
            FENString::try_parse_castle_rights(castle_rights_field)
                .expect("Parsing valid castle rights field should not throw an error"),
            expected
        );

        let castle_rights_field = "KQkq";
        let expected = [CastleRights::KingQueen, CastleRights::KingQueen];
        assert_eq!(
            FENString::try_parse_castle_rights(castle_rights_field)
                .expect("Parsing valid castle rights field should not throw an error"),
            expected
        );

        let castle_rights_field = "-";
        let expected = [CastleRights::Neither, CastleRights::Neither];
        assert_eq!(
            FENString::try_parse_castle_rights(castle_rights_field)
                .expect("Parsing valid castle rights field should not throw an error"),
            expected
        );

        let castle_rights_field = "KQkqq";
        assert!(FENString::try_parse_castle_rights(castle_rights_field).is_err());

        let castle_rights_field = "WQkq";
        assert!(FENString::try_parse_castle_rights(castle_rights_field).is_err());

        let castle_rights_field = "-K";
        assert!(FENString::try_parse_castle_rights(castle_rights_field).is_err());

        // Castle rights should follow 'King Queen' order
        let castle_rights_field = "QKqk";
        assert!(FENString::try_parse_castle_rights(castle_rights_field).is_err());

        let castle_rights_field = "KkQq";
        assert!(FENString::try_parse_castle_rights(castle_rights_field).is_err());

        let castle_rights_field = "KKQQ";
        assert!(FENString::try_parse_castle_rights(castle_rights_field).is_err());

        let castle_rights_field = "QK";
        assert!(FENString::try_parse_castle_rights(castle_rights_field).is_err());

        let castle_rights_field = "";
        assert!(FENString::try_parse_castle_rights(castle_rights_field).is_err());
    }
}
