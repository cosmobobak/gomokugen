use std::{
    fmt::{Debug, Display},
    hash::Hash,
    str::FromStr,
};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Player {
    /// Neither player has a piece on this square.
    None,
    /// The first player.
    X,
    /// The second player.
    O,
}

impl std::ops::Neg for Player {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Self::X => Self::O,
            Self::O => Self::X,
            Self::None => panic!("No player to move"),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Move<const SIDE_LENGTH: usize> {
    index: u16,
}

impl<const SIDE_LENGTH: usize> Move<SIDE_LENGTH> {
    #[must_use]
    pub const fn null() -> Self {
        Self { index: u16::MAX }
    }

    #[must_use]
    pub const fn is_null(&self) -> bool {
        self.index == u16::MAX
    }

    #[must_use]
    pub const fn index(&self) -> usize {
        self.index as usize
    }
}

impl<const SIDE_LENGTH: usize> Display for Move<SIDE_LENGTH> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let row = self.index / SIDE_LENGTH as u16;
        let col = self.index % SIDE_LENGTH as u16;
        write!(
            f,
            "{}{}",
            (b'A' + u8::try_from(row).unwrap()) as char,
            col + 1
        )
    }
}

impl<const SIDE_LENGTH: usize> Debug for Move<SIDE_LENGTH> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let row = self.index / SIDE_LENGTH as u16;
        let col = self.index % SIDE_LENGTH as u16;
        write!(
            f,
            "{}{} ({})",
            (b'A' + u8::try_from(row).unwrap()) as char,
            col + 1,
            self.index
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Board<const SIDE_LENGTH: usize> {
    cells: [[Player; SIDE_LENGTH]; SIDE_LENGTH],
    last_move: Option<Move<SIDE_LENGTH>>,
    ply: u16,
}

impl<const SIDE_LENGTH: usize> PartialEq for Board<SIDE_LENGTH> {
    fn eq(&self, other: &Self) -> bool {
        self.cells == other.cells
    }
}

impl<const SIDE_LENGTH: usize> Eq for Board<SIDE_LENGTH> {}

impl<const SIDE_LENGTH: usize> Hash for Board<SIDE_LENGTH> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.cells.hash(state);
    }
}

/// A gomoku board of size `SIDE_LENGTH` by `SIDE_LENGTH`.
impl<const SIDE_LENGTH: usize> Board<SIDE_LENGTH> {
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    const N_I: isize = SIDE_LENGTH as isize;

    /// Creates a new board with no pieces on it.
    ///
    /// # Panics
    ///
    /// Panics if `SIDE_LENGTH` is greater than 19.
    #[must_use]
    pub fn new() -> Self {
        assert!(
            SIDE_LENGTH <= 19,
            "Only boards of up to 19x19 are supported."
        );
        Self {
            cells: [[Player::None; SIDE_LENGTH]; SIDE_LENGTH],
            last_move: None,
            ply: 0,
        }
    }

    /// Generates all possible moves on the board and calls `callback` with each one.
    /// Iteration short-circuits if `callback` returns `true`.
    pub fn generate_moves(&self, mut callback: impl FnMut(Move<SIDE_LENGTH>) -> bool) {
        #![allow(clippy::cast_possible_truncation)]
        for (i, c) in self.cells.iter().flatten().enumerate() {
            if *c == Player::None && callback(Move { index: i as u16 }) {
                return;
            }
        }
    }

    /// Iterates over all filled cells on the board and calls `callback` with each one.
    pub fn feature_map(&self, mut callback: impl FnMut(usize, Player)) {
        for (i, c) in self.cells.iter().flatten().enumerate() {
            if *c != Player::None {
                callback(i, *c);
            }
        }
    }

    /// Applies a move to the board.
    pub fn make_move(&mut self, mv @ Move { index }: Move<SIDE_LENGTH>) {
        #![allow(clippy::cast_possible_truncation)]
        debug_assert!(!mv.is_null(), "Cannot make null move");
        let i = (index / SIDE_LENGTH as u16) as usize;
        let j = (index % SIDE_LENGTH as u16) as usize;
        self.cells[i][j] = self.turn();
        self.last_move = Some(mv);
        self.ply += 1;
    }

    /// Returns the player whose turn it is.
    #[must_use]
    pub const fn turn(&self) -> Player {
        match self.ply % 2 {
            0 => Player::X,
            _ => Player::O,
        }
    }

    fn row_along<const D_X: isize, const D_Y: isize>(&self, row: usize, col: usize) -> bool {
        #![allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
        let mut count = 1;
        let last_piece = -self.turn();

        if !(D_X < 0 && row == 0
            || D_Y < 0 && col == 0
            || D_X > 0 && row == SIDE_LENGTH - 1
            || D_Y > 0 && col == SIDE_LENGTH - 1)
        {
            let mut row_u = row as isize + D_X;
            let mut col_u = col as isize + D_Y;
            loop {
                // count pieces in a direction until we hit a piece of the opposite color or an empty space
                if self.cells[row_u as usize][col_u as usize] != last_piece {
                    break;
                }
                count += 1;
                if count == 5 {
                    return true;
                }
                if D_X < 0 && row_u == 0
                    || D_Y < 0 && col_u == 0
                    || D_X > 0 && row_u == Self::N_I - 1
                    || D_Y > 0 && col_u == Self::N_I - 1
                {
                    break;
                }
                row_u += D_X;
                col_u += D_Y;
            }
        }
        if !(D_X > 0 && row == 0
            || D_Y > 0 && col == 0
            || D_X < 0 && row == SIDE_LENGTH - 1
            || D_Y < 0 && col == SIDE_LENGTH - 1)
        {
            let mut row_d = row as isize - D_X;
            let mut col_d = col as isize - D_Y;
            loop {
                // count pieces in -direction until we hit a piece of the opposite color or an empty space
                if self.cells[row_d as usize][col_d as usize] != last_piece {
                    break;
                }
                count += 1;
                if count == 5 {
                    return true;
                }
                if D_X > 0 && row_d == 0
                    || D_Y > 0 && col_d == 0
                    || D_X < 0 && row_d == Self::N_I - 1
                    || D_Y < 0 && col_d == Self::N_I - 1
                {
                    break;
                }
                row_d -= D_X;
                col_d -= D_Y;
            }
        }

        false
    }

    /// Returns the outcome of the game, if any.
    ///
    /// `None` means the game is still in progress.
    /// `Some(Player::None)` means the game is a draw.
    #[must_use]
    pub fn outcome(&self) -> Option<Player> {
        #![allow(clippy::cast_possible_truncation)]
        let Move { index } = self.last_move?;
        let row = (index / SIDE_LENGTH as u16) as usize;
        let col = (index % SIDE_LENGTH as u16) as usize;

        if self.row_along::<0, 1>(row, col)
            || self.row_along::<1, 0>(row, col)
            || self.row_along::<1, 1>(row, col)
            || self.row_along::<1, -1>(row, col)
        {
            return Some(-self.turn());
        }

        if self.ply as usize == SIDE_LENGTH * SIDE_LENGTH {
            Some(Player::None)
        } else {
            None
        }
    }

    /// The FEN string for the current board state.
    #[must_use]
    pub fn fen(&self) -> String {
        let mut out = String::new();
        for row in &self.cells {
            let mut count = 0;
            for c in row {
                match c {
                    Player::None => out.push('.'),
                    Player::X => out.push('x'),
                    Player::O => out.push('o'),
                }
                count += 1;
            }
            assert!(count == SIDE_LENGTH, "Invalid board state");
            out.push('/');
        }
        out.pop();
        out.push(' ');
        out.push(match self.turn() {
            Player::X => 'x',
            Player::O => 'o',
            Player::None => panic!("No player to move"),
        });
        out.push(' ');
        out.push_str(&self.ply.to_string());
        out
    }
}

impl<const SIDE_LENGTH: usize> Default for Board<SIDE_LENGTH> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const SIDE_LENGTH: usize> Display for Board<SIDE_LENGTH> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut row = 0;
        for (i, c) in self.cells.iter().flatten().enumerate() {
            if i % SIDE_LENGTH == 0 {
                write!(f, "{} ", (b'A' + row as u8) as char)?;
                row += 1;
            }
            write!(
                f,
                "{} ",
                match c {
                    Player::None => '.',
                    Player::X => 'X',
                    Player::O => 'O',
                }
            )?;
            if i % SIDE_LENGTH == SIDE_LENGTH - 1 {
                writeln!(f)?;
            }
        }
        write!(f, "  ")?;
        for i in 0..SIDE_LENGTH {
            write!(f, "{} ", i + 1)?;
        }
        writeln!(f)
    }
}

impl<const SIDE_LENGTH: usize> FromStr for Board<SIDE_LENGTH> {
    type Err = &'static str;

    /// Parses a FEN string variant for gomoku.
    /// an example 7x7 fen string would be:
    /// `x......o/......../......../......../......../......../o......x x 4`,
    /// meaning that there are four pieces placed (in the corners)
    /// and x is to move next.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut out = Self::new();
        let mut parts = s.split_whitespace();
        let Some(rows) = parts.next().map(|s| s.split('/')) else {
            return Err("No board part found in FEN string");
        };
        let Some(turn) = parts.next().and_then(|s| s.chars().next()) else {
            return Err("No turn part found in FEN string");
        };
        let turn = match turn {
            'x' => Player::X,
            'o' => Player::O,
            _ => return Err("Invalid turn part found in FEN string"),
        };
        let Some(ply) = parts.next().and_then(|s| s.parse::<u16>().ok()) else {
            return Err("No ply part found in FEN string");
        };
        out.ply = ply;
        if out.turn() != turn {
            return Err("Turn part does not match ply part in FEN string");
        }
        for (i, row) in rows.enumerate() {
            let mut col = 0;
            for c in row.chars() {
                if col >= SIDE_LENGTH {
                    return Err("Too many columns in FEN string");
                }
                match c {
                    'x' => out.cells[i][col] = Player::X,
                    'o' => out.cells[i][col] = Player::O,
                    '.' => out.cells[i][col] = Player::None,
                    _ => return Err("Invalid character in FEN string"),
                }
                col += 1;
            }
            if col != SIDE_LENGTH {
                return Err("Too few columns in FEN string");
            }
        }
        Ok(out)
    }
}

mod tests {
    #[test]
    fn first_player_is_x() {
        use super::*;
        let board = Board::<19>::new();
        assert_eq!(board.turn(), Player::X);
    }

    #[test]
    fn second_player_is_o() {
        use super::*;
        let mut board = Board::<19>::new();
        board.make_move(Move { index: 0 });
        assert_eq!(board.turn(), Player::O);
    }

    #[test]
    fn fen_string_round_trip_startpos() {
        use super::*;
        let board = Board::<19>::new();
        let fen = board.fen();
        let board2 = Board::<19>::from_str(&fen).unwrap();
        assert_eq!(board, board2);
    }

    #[test]
    fn fen_string_round_trip_7x7() {
        use super::*;
        let mut board = Board::<7>::new();
        board.make_move(Move { index: 0 });
        board.make_move(Move { index: 48 });
        let fen = board.fen();
        let board2 = Board::<7>::from_str(&fen).unwrap();
        assert_eq!(board, board2);
    }

    #[test]
    fn fen_string_round_trip_19x19() {
        use super::*;
        let mut board = Board::<19>::new();
        board.make_move(Move { index: 0 });
        board.make_move(Move { index: 360 });
        let fen = board.fen();
        let board2 = Board::<19>::from_str(&fen).unwrap();
        assert_eq!(board, board2);
    }

    #[test]
    fn fen_string_round_trip_alt() {
        use super::*;
        let fen = "x.....o/......./......./......./......./......./o.....x x 4";
        let board = Board::<7>::from_str(fen).unwrap();
        let fen2 = board.fen();
        assert_eq!(fen, fen2);
    }
}
