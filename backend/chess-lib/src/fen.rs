use anyhow::{Context, anyhow};
use strum::EnumCount;

use crate::{
    CastleRights, GameStats, PieceType, Side, Square,
    square::{File, Rank, TryNext, TryPrevious},
};

/// A format to store chess positions in an easily parsable string.
///
/// See <https://www.chess.com/terms/fen_str-chess> for details.
struct FENString {
    piece_positions: [[Vec<Square>; PieceType::COUNT]; Side::COUNT],
    game_stats: GameStats,
}

impl FENString {
    fn try_parse(fen_str: &str) -> anyhow::Result<Self> {
        let mut fields = fen_str.split_ascii_whitespace();

        let position = fields.next().context("FEN string is empty")?;
        let piece_positions = Self::try_parse_position(position)?;

        let active_color = fields.next().context("FEN string is incomplete")?;
        let active_color = Self::try_parse_active_color(active_color)?;

        let castle_rights = fields.next().context("FEN string is incomplete")?;
        let castle_rights = Self::try_parse_castle_rights(castle_rights)?;

        let en_passant_target = fields.next().context("FEN string is incomplete")?;
        let en_passant_target = Self::try_parse_en_passant_target(en_passant_target)?;

        let halfmoves = fields.next().context("FEN string is incomplete")?;
        let halfmoves = Self::try_parse_halfmoves(halfmoves)?;

        let fullmoves = fields.next().context("FEN string is incomplete")?;
        let fullmoves = Self::try_parse_fullmoves(fullmoves)?;

        let game_stats = GameStats {
            active_color,
            castle_rights,
            en_passant_target,
            halfmoves,
            fullmoves,
        };

        Ok(Self {
            piece_positions,
            game_stats,
        })
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

    fn try_parse_en_passant_target(en_passant_target: &str) -> anyhow::Result<Option<Square>> {
        if en_passant_target == "-" {
            return Ok(None);
        }

        Ok(Some(Square::try_from(en_passant_target)?))
    }

    fn try_parse_halfmoves(halfmoves: &str) -> anyhow::Result<u8> {
        let halfmoves = halfmoves
            .parse::<u8>()
            .with_context(|| format!("FEN halfmoves field is invalid: {halfmoves}"))?;

        // Enforce the 50-move rule
        if halfmoves > 100 {
            return Err(anyhow!(
                "Number of halfmoves exceeds the 50-move rule: {halfmoves}"
            ));
        }

        Ok(halfmoves)
    }

    fn try_parse_fullmoves(fullmoves: &str) -> anyhow::Result<u16> {
        let fullmoves = fullmoves
            .parse::<u16>()
            .with_context(|| format!("FEN fullmoves field is invalid: {fullmoves}"))?;

        if fullmoves == 0 {
            return Err(anyhow!("Number of fullmoves cannot equal 0: {fullmoves}"));
        }

        Ok(fullmoves)
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

        // White should have 8 pawns on the 2nd rank
        assert_eq!(fen_str.piece_positions[white_idx][pawn_idx].len(), 8);
        assert!(fen_str.piece_positions[white_idx][pawn_idx].contains(&Square::A2));
        assert!(fen_str.piece_positions[white_idx][pawn_idx].contains(&Square::H2));

        // White should have rooks on A1 and H1
        assert_eq!(fen_str.piece_positions[white_idx][rook_idx].len(), 2);
        assert!(fen_str.piece_positions[white_idx][rook_idx].contains(&Square::A1));
        assert!(fen_str.piece_positions[white_idx][rook_idx].contains(&Square::H1));

        // Black should have 8 pawns on the 7th rank
        assert_eq!(fen_str.piece_positions[black_idx][pawn_idx].len(), 8);
        assert!(fen_str.piece_positions[black_idx][pawn_idx].contains(&Square::A7));
        assert!(fen_str.piece_positions[black_idx][pawn_idx].contains(&Square::H7));

        // Black should have rooks on A8 and H8
        assert_eq!(fen_str.piece_positions[black_idx][rook_idx].len(), 2);
        assert!(fen_str.piece_positions[black_idx][rook_idx].contains(&Square::A8));
        assert!(fen_str.piece_positions[black_idx][rook_idx].contains(&Square::H8));

        // Make sure cross-contamination didn't happen (e.g., White pawns on Black's side)
        assert!(!fen_str.piece_positions[white_idx][pawn_idx].contains(&Square::A7));
        assert!(!fen_str.piece_positions[black_idx][pawn_idx].contains(&Square::A2));

        let expected_stats = GameStats {
            active_color: Side::White,
            castle_rights: [CastleRights::KingQueen, CastleRights::KingQueen],
            en_passant_target: None,
            halfmoves: 0,
            fullmoves: 1,
        };
        assert_eq!(fen_str.game_stats, expected_stats);
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

    #[test]
    fn en_passant_target() {
        // --- Valid Targets ---
        // An en passant target square on e3 (White just moved e2-e4)
        let target = "e3";
        let expected = Some(Square::E3);
        assert_eq!(
            FENString::try_parse_en_passant_target(target)
                .expect("Parsing 'e3' en passant target should succeed"),
            expected
        );

        // An en passant target square on c6 (Black just moved c7-c5)
        let target = "c6";
        let expected = Some(Square::C6);
        assert_eq!(
            FENString::try_parse_en_passant_target(target)
                .expect("Parsing 'c6' en passant target should succeed"),
            expected
        );

        // --- Empty State ---
        // No en passant target available (the most common state, represented by '-')
        let target = "-";
        let expected = None;
        assert_eq!(
            FENString::try_parse_en_passant_target(target)
                .expect("Parsing '-' en passant target should return None safely"),
            expected
        );

        // --- Invalid Bounds & Formats ---
        // Completely invalid file/rank characters
        let target = "z3";
        assert!(FENString::try_parse_en_passant_target(target).is_err());

        // Valid file but completely out-of-bounds rank for chess coordinates
        let target = "e9";
        assert!(FENString::try_parse_en_passant_target(target).is_err());

        // Field string is too long to be a single coordinate square
        let target = "e3e4";
        assert!(FENString::try_parse_en_passant_target(target).is_err());

        // Field string is empty
        let target = "";
        assert!(FENString::try_parse_en_passant_target(target).is_err());
    }

    #[test]
    fn halfmoves() {
        assert_eq!(
            FENString::try_parse_halfmoves("0").expect("Should parse 0 halfmoves"),
            0
        );
        assert_eq!(
            FENString::try_parse_halfmoves("14").expect("Should parse 14 halfmoves"),
            14
        );

        // The 50-move rule claim threshold
        assert_eq!(
            FENString::try_parse_halfmoves("100").expect("Should parse 100 halfmoves"),
            100
        );

        // Invalid Boundary Violations
        assert!(FENString::try_parse_halfmoves("151").is_err());
        assert!(FENString::try_parse_halfmoves("200").is_err());

        assert!(FENString::try_parse_halfmoves("-5").is_err());
        assert!(FENString::try_parse_halfmoves("12.5").is_err());
        assert!(FENString::try_parse_halfmoves("abc").is_err());
        assert!(FENString::try_parse_halfmoves("").is_err());
    }

    #[test]
    fn fullmoves() {
        assert_eq!(
            FENString::try_parse_fullmoves("1").expect("Should parse fullmove 1"),
            1
        );

        assert_eq!(
            FENString::try_parse_fullmoves("45").expect("Should parse fullmove 45"),
            45
        );

        assert_eq!(
            FENString::try_parse_fullmoves("350").expect("Should parse fullmove 350"),
            350
        );

        // A fullmove count of 0 is physically impossible in chess notation
        assert!(FENString::try_parse_fullmoves("0").is_err());

        // Value exceeds a u16
        assert!(FENString::try_parse_fullmoves("70000").is_err());

        assert!(FENString::try_parse_fullmoves("-1").is_err());
        assert!(FENString::try_parse_fullmoves("twelve").is_err());
        assert!(FENString::try_parse_fullmoves("").is_err());
    }
}
