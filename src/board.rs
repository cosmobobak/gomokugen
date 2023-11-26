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
pub struct Move {
    index: u16,
}

impl Move {
    #[must_use]
    pub const fn null() -> Self {
        Self { index: u16::MAX }
    }

    #[must_use]
    pub const fn is_null(&self) -> bool {
        self.index == u16::MAX
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Board<const SIDE_LENGTH: usize> {
    cells: [[Player; SIDE_LENGTH]; SIDE_LENGTH],
    last_move: Option<Move>,
    ply: u16,
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
        assert!(SIDE_LENGTH <= 19, "Only boards of up to 19x19 are supported.");
        Self {
            cells: [[Player::None; SIDE_LENGTH]; SIDE_LENGTH],
            last_move: None,
            ply: 0,
        }
    }

    /// Generates all possible moves on the board and calls `callback` with each one.
    pub fn generate_moves(&self, mut callback: impl FnMut(Move) -> bool) {
        #![allow(clippy::cast_possible_truncation)]
        for (i, c) in self.cells.iter().flatten().enumerate() {
            if *c == Player::None && callback(Move { index: i as u16 }) {
                return;
            }
        }
    }

    /// Applies a move to the board.
    pub fn make_move(&mut self, mv @ Move { index }: Move) {
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
}

impl<const SIDE_LENGTH: usize> Default for Board<SIDE_LENGTH> {
    fn default() -> Self {
        Self::new()
    }
}

mod tests {
    use super::*;

    #[test]
    fn first_player_is_x() {
        let board = Board::<19>::new();
        assert_eq!(board.turn(), Player::X);
    }
}