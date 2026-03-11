// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Clone, PartialEq, Debug)]
pub enum CellState {
    Dead,
    Alive,
}

#[allow(dead_code)]
pub struct CellAutomaton {
    pub cells: Vec<CellState>,
    pub width: usize,
    pub height: usize,
}

#[allow(dead_code)]
pub fn new_cell_automaton(w: usize, h: usize) -> CellAutomaton {
    let cells = vec![CellState::Dead; w * h];
    CellAutomaton { cells, width: w, height: h }
}

#[allow(dead_code)]
pub fn ca_set(g: &mut CellAutomaton, x: usize, y: usize, state: CellState) {
    g.cells[y * g.width + x] = state;
}

#[allow(dead_code)]
pub fn ca_get(g: &CellAutomaton, x: usize, y: usize) -> &CellState {
    &g.cells[y * g.width + x]
}

#[allow(dead_code)]
pub fn ca_alive_count(g: &CellAutomaton) -> usize {
    g.cells.iter().filter(|c| **c == CellState::Alive).count()
}

#[allow(dead_code)]
pub fn ca_step_conway(g: &mut CellAutomaton) {
    let w = g.width;
    let h = g.height;
    let old = g.cells.clone();
    for y in 0..h {
        for x in 0..w {
            let mut neighbors = 0usize;
            for dy in 0i32..=2 {
                for dx in 0i32..=2 {
                    if dx == 1 && dy == 1 {
                        continue;
                    }
                    let nx = x as i32 + dx - 1;
                    let ny = y as i32 + dy - 1;
                    if nx >= 0 && ny >= 0 && (nx as usize) < w && (ny as usize) < h
                        && old[ny as usize * w + nx as usize] == CellState::Alive
                    {
                        neighbors += 1;
                    }
                }
            }
            let idx = y * w + x;
            g.cells[idx] = match (&old[idx], neighbors) {
                (CellState::Alive, 2) | (CellState::Alive, 3) => CellState::Alive,
                (CellState::Dead, 3) => CellState::Alive,
                _ => CellState::Dead,
            };
        }
    }
}

#[allow(dead_code)]
pub fn ca_clear(g: &mut CellAutomaton) {
    for c in g.cells.iter_mut() {
        *c = CellState::Dead;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_all_dead() {
        let g = new_cell_automaton(4, 4);
        assert_eq!(ca_alive_count(&g), 0);
    }

    #[test]
    fn test_set_get() {
        let mut g = new_cell_automaton(3, 3);
        ca_set(&mut g, 1, 1, CellState::Alive);
        assert_eq!(ca_get(&g, 1, 1), &CellState::Alive);
        assert_eq!(ca_get(&g, 0, 0), &CellState::Dead);
    }

    #[test]
    fn test_alive_count() {
        let mut g = new_cell_automaton(4, 4);
        ca_set(&mut g, 0, 0, CellState::Alive);
        ca_set(&mut g, 1, 1, CellState::Alive);
        assert_eq!(ca_alive_count(&g), 2);
    }

    #[test]
    fn test_step_conway_no_crash() {
        let mut g = new_cell_automaton(5, 5);
        ca_set(&mut g, 2, 1, CellState::Alive);
        ca_set(&mut g, 2, 2, CellState::Alive);
        ca_set(&mut g, 2, 3, CellState::Alive);
        ca_step_conway(&mut g);
        assert!(ca_alive_count(&g) <= 25);
    }

    #[test]
    fn test_clear() {
        let mut g = new_cell_automaton(3, 3);
        ca_set(&mut g, 0, 0, CellState::Alive);
        ca_clear(&mut g);
        assert_eq!(ca_alive_count(&g), 0);
    }

    #[test]
    fn test_blinker_oscillator() {
        let mut g = new_cell_automaton(5, 5);
        ca_set(&mut g, 2, 1, CellState::Alive);
        ca_set(&mut g, 2, 2, CellState::Alive);
        ca_set(&mut g, 2, 3, CellState::Alive);
        ca_step_conway(&mut g);
        assert_eq!(ca_alive_count(&g), 3);
    }

    #[test]
    fn test_isolated_cell_dies() {
        let mut g = new_cell_automaton(5, 5);
        ca_set(&mut g, 2, 2, CellState::Alive);
        ca_step_conway(&mut g);
        assert_eq!(ca_alive_count(&g), 0);
    }

    #[test]
    fn test_width_height() {
        let g = new_cell_automaton(6, 8);
        assert_eq!(g.width, 6);
        assert_eq!(g.height, 8);
    }

    #[test]
    fn test_grid_size() {
        let g = new_cell_automaton(3, 4);
        assert_eq!(g.cells.len(), 12);
    }
}
