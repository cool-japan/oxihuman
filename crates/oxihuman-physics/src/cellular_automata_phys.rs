// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/* Cell types: 0=empty, 1=sand, 2=water, 3=wall */
pub const CELL_EMPTY: u8 = 0;
pub const CELL_SAND: u8 = 1;
pub const CELL_WATER: u8 = 2;
pub const CELL_WALL: u8 = 3;

pub struct CaSandGrid {
    pub cells: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

pub fn new_ca_sand_grid(w: usize, h: usize) -> CaSandGrid {
    CaSandGrid {
        cells: vec![CELL_EMPTY; w * h],
        width: w,
        height: h,
    }
}

pub fn ca_set(g: &mut CaSandGrid, x: usize, y: usize, cell: u8) {
    if x < g.width && y < g.height {
        g.cells[y * g.width + x] = cell;
    }
}

pub fn ca_get(g: &CaSandGrid, x: usize, y: usize) -> u8 {
    if x < g.width && y < g.height {
        g.cells[y * g.width + x]
    } else {
        CELL_WALL
    }
}

pub fn ca_step(g: &mut CaSandGrid) {
    /* process from bottom to top so sand falls correctly */
    let w = g.width;
    let h = g.height;
    let mut next = g.cells.clone();
    for y in (0..h).rev() {
        for x in 0..w {
            let cell = g.cells[y * w + x];
            if cell == CELL_SAND && y + 1 < h {
                let below = g.cells[(y + 1) * w + x];
                if below == CELL_EMPTY {
                    next[y * w + x] = CELL_EMPTY;
                    next[(y + 1) * w + x] = CELL_SAND;
                }
            }
        }
    }
    g.cells = next;
}

pub fn ca_count_material(g: &CaSandGrid, material: u8) -> usize {
    g.cells.iter().filter(|&&c| c == material).count()
}

pub fn ca_is_settled(g: &CaSandGrid) -> bool {
    /* settled when no sand has empty below it */
    let w = g.width;
    let h = g.height;
    for y in 0..h.saturating_sub(1) {
        for x in 0..w {
            if g.cells[y * w + x] == CELL_SAND && g.cells[(y + 1) * w + x] == CELL_EMPTY {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_ca_sand_grid() {
        /* new grid is all empty */
        let g = new_ca_sand_grid(4, 4);
        assert_eq!(ca_count_material(&g, CELL_EMPTY), 16);
    }

    #[test]
    fn test_ca_set_get() {
        /* set and get cell value */
        let mut g = new_ca_sand_grid(4, 4);
        ca_set(&mut g, 2, 1, CELL_SAND);
        assert_eq!(ca_get(&g, 2, 1), CELL_SAND);
    }

    #[test]
    fn test_ca_step_sand_falls() {
        /* sand at row 0 falls to row 1 when row 1 is empty */
        let mut g = new_ca_sand_grid(3, 3);
        ca_set(&mut g, 1, 0, CELL_SAND);
        ca_step(&mut g);
        assert_eq!(ca_get(&g, 1, 1), CELL_SAND);
        assert_eq!(ca_get(&g, 1, 0), CELL_EMPTY);
    }

    #[test]
    fn test_ca_sand_settles_on_bottom() {
        /* sand falls to bottom and settles */
        let mut g = new_ca_sand_grid(3, 5);
        ca_set(&mut g, 1, 0, CELL_SAND);
        for _ in 0..10 {
            ca_step(&mut g);
        }
        assert_eq!(ca_get(&g, 1, 4), CELL_SAND);
        assert!(ca_is_settled(&g));
    }

    #[test]
    fn test_ca_count_material() {
        /* count specific material type */
        let mut g = new_ca_sand_grid(4, 4);
        ca_set(&mut g, 0, 0, CELL_SAND);
        ca_set(&mut g, 1, 0, CELL_SAND);
        ca_set(&mut g, 2, 0, CELL_WALL);
        assert_eq!(ca_count_material(&g, CELL_SAND), 2);
        assert_eq!(ca_count_material(&g, CELL_WALL), 1);
    }

    #[test]
    fn test_ca_is_settled_empty_grid() {
        /* empty grid is settled */
        let g = new_ca_sand_grid(4, 4);
        assert!(ca_is_settled(&g));
    }

    #[test]
    fn test_ca_sand_blocked_by_wall() {
        /* sand stops on top of wall */
        let mut g = new_ca_sand_grid(3, 4);
        ca_set(&mut g, 1, 3, CELL_WALL);
        ca_set(&mut g, 1, 0, CELL_SAND);
        for _ in 0..10 {
            ca_step(&mut g);
        }
        /* sand should be above wall */
        assert_eq!(ca_get(&g, 1, 2), CELL_SAND);
    }
}
