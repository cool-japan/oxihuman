// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! 2D prefix sum table for O(1) rectangular range queries.

pub struct PrefixSum2d {
    pub data: Vec<i64>,
    pub rows: usize,
    pub cols: usize,
}

pub fn new_prefix_sum_2d(grid: &[Vec<i64>]) -> PrefixSum2d {
    let rows = grid.len();
    let cols = if rows > 0 { grid[0].len() } else { 0 };
    // data[i][j] = prefix sum of rectangle [0..i][0..j] (1-indexed)
    let mut data = vec![0i64; (rows + 1) * (cols + 1)];
    let idx = |r: usize, c: usize| r * (cols + 1) + c;
    for r in 1..=rows {
        for c in 1..=cols {
            let v = grid[r - 1].get(c - 1).copied().unwrap_or(0);
            data[idx(r, c)] =
                v + data[idx(r - 1, c)] + data[idx(r, c - 1)] - data[idx(r - 1, c - 1)];
        }
    }
    PrefixSum2d { data, rows, cols }
}

pub fn ps2d_query(t: &PrefixSum2d, r1: usize, c1: usize, r2: usize, c2: usize) -> i64 {
    if t.rows == 0 || t.cols == 0 {
        return 0;
    }
    let cols1 = t.cols + 1;
    let idx = |r: usize, c: usize| r * cols1 + c;
    t.data[idx(r2 + 1, c2 + 1)] - t.data[idx(r1, c2 + 1)] - t.data[idx(r2 + 1, c1)]
        + t.data[idx(r1, c1)]
}

pub fn ps2d_rows(t: &PrefixSum2d) -> usize {
    t.rows
}

pub fn ps2d_cols(t: &PrefixSum2d) -> usize {
    t.cols
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_grid() -> Vec<Vec<i64>> {
        vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]]
    }

    #[test]
    fn test_full_sum() {
        /* sum of full grid is 45 */
        let t = new_prefix_sum_2d(&make_grid());
        assert_eq!(ps2d_query(&t, 0, 0, 2, 2), 45);
    }

    #[test]
    fn test_single_cell() {
        /* single cell query returns that cell's value */
        let t = new_prefix_sum_2d(&make_grid());
        assert_eq!(ps2d_query(&t, 1, 1, 1, 1), 5);
    }

    #[test]
    fn test_subgrid() {
        /* 2x2 subgrid sum */
        let t = new_prefix_sum_2d(&make_grid());
        assert_eq!(ps2d_query(&t, 0, 0, 1, 1), 12);
    }

    #[test]
    fn test_rows_cols() {
        /* rows and cols match input dimensions */
        let t = new_prefix_sum_2d(&make_grid());
        assert_eq!(ps2d_rows(&t), 3);
        assert_eq!(ps2d_cols(&t), 3);
    }

    #[test]
    fn test_single_row() {
        /* single row grid works correctly */
        let grid = vec![vec![1i64, 2, 3, 4]];
        let t = new_prefix_sum_2d(&grid);
        assert_eq!(ps2d_query(&t, 0, 1, 0, 2), 5);
    }

    #[test]
    fn test_empty_grid() {
        /* empty grid returns 0 */
        let t = new_prefix_sum_2d(&[]);
        assert_eq!(ps2d_rows(&t), 0);
        assert_eq!(ps2d_cols(&t), 0);
    }
}
