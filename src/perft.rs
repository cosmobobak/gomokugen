use std::{collections::HashMap, hash::BuildHasher};

use crate::board::Board;

#[must_use]
pub fn perft<const BOARD_SIZE: usize>(board: Board<BOARD_SIZE>, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }

    if depth == 1 {
        let mut count = 0;
        board.generate_moves(|_| {
            count += 1;
            false
        });
        return count;
    }

    let mut count = 0;
    board.generate_moves(|mv| {
        let mut board = board;
        board.make_move(mv);
        count += perft(board, depth - 1);
        false
    });

    count
}

#[allow(clippy::module_name_repetitions)]
#[must_use]
pub fn perft_cached<const BOARD_SIZE: usize, S: BuildHasher>(
    board: Board<BOARD_SIZE>,
    depth: u8,
    cache: &mut HashMap<(Board<BOARD_SIZE>, u8), u64, S>,
) -> u64 {
    if depth == 0 {
        return 1;
    }

    if depth == 1 {
        let mut count = 0;
        board.generate_moves(|_| {
            count += 1;
            false
        });
        return count;
    }

    if let Some(&count) = cache.get(&(board, depth)) {
        return count;
    }

    let mut count = 0;
    board.generate_moves(|mv| {
        let mut board = board;
        board.make_move(mv);
        count += perft_cached(board, depth - 1, cache);
        false
    });

    cache.insert((board, depth), count);

    count
}