// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Peridynamics bond-based model stub — replaces local stress-strain with
//! pairwise bond forces acting within a neighbourhood horizon δ.

/// A peridynamic material point.
#[derive(Debug, Clone)]
pub struct PdPoint {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub mass: f64,
    pub broken: bool,
}

impl PdPoint {
    pub fn new(x: f64, y: f64, mass: f64) -> Self {
        Self {
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            mass,
            broken: false,
        }
    }
}

/// A bond between two points.
#[derive(Debug, Clone)]
pub struct Bond {
    pub i: usize,
    pub j: usize,
    pub rest_length: f64,
    pub broken: bool,
}

/// Peridynamic simulation.
pub struct Peridynamics {
    pub points: Vec<PdPoint>,
    pub bonds: Vec<Bond>,
    pub horizon: f64,
    pub stiffness: f64,
    pub critical_stretch: f64,
}

impl Peridynamics {
    /// Create a new peridynamic model.
    pub fn new(horizon: f64, stiffness: f64, critical_stretch: f64) -> Self {
        Self {
            points: Vec::new(),
            bonds: Vec::new(),
            horizon,
            stiffness,
            critical_stretch,
        }
    }

    /// Add a point.
    pub fn add_point(&mut self, x: f64, y: f64, mass: f64) -> usize {
        let idx = self.points.len();
        self.points.push(PdPoint::new(x, y, mass));
        idx
    }

    /// Automatically create bonds between all pairs within the horizon.
    pub fn build_bonds(&mut self) {
        let n = self.points.len();
        for i in 0..n {
            for j in (i + 1)..n {
                let dx = self.points[j].x - self.points[i].x;
                let dy = self.points[j].y - self.points[i].y;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist <= self.horizon {
                    self.bonds.push(Bond {
                        i,
                        j,
                        rest_length: dist,
                        broken: false,
                    });
                }
            }
        }
    }

    /// Compute and apply bond forces; break bonds beyond critical stretch.
    pub fn apply_forces(&mut self) {
        let k = self.stiffness;
        let crit = self.critical_stretch;
        let mut forces_x = vec![0.0f64; self.points.len()];
        let mut forces_y = vec![0.0f64; self.points.len()];
        for bond in &mut self.bonds {
            if bond.broken {
                continue;
            }
            let dx = self.points[bond.j].x - self.points[bond.i].x;
            let dy = self.points[bond.j].y - self.points[bond.i].y;
            let dist = (dx * dx + dy * dy).sqrt();
            if bond.rest_length > 0.0 {
                let stretch = (dist - bond.rest_length) / bond.rest_length;
                if stretch.abs() > crit {
                    bond.broken = true;
                    continue;
                }
                if dist > 1e-14 {
                    let f = k * stretch;
                    forces_x[bond.i] += f * dx / dist;
                    forces_y[bond.i] += f * dy / dist;
                    forces_x[bond.j] -= f * dx / dist;
                    forces_y[bond.j] -= f * dy / dist;
                }
            }
        }
        for (i, p) in self.points.iter_mut().enumerate() {
            if p.mass > 0.0 {
                p.vx += forces_x[i] / p.mass;
                p.vy += forces_y[i] / p.mass;
            }
        }
    }

    /// Integrate positions by dt.
    pub fn integrate(&mut self, dt: f64) {
        for p in &mut self.points {
            p.x += p.vx * dt;
            p.y += p.vy * dt;
        }
    }

    /// Number of active (not broken) bonds.
    pub fn active_bond_count(&self) -> usize {
        self.bonds.iter().filter(|b| !b.broken).count()
    }

    /// Number of broken bonds.
    pub fn broken_bond_count(&self) -> usize {
        self.bonds.iter().filter(|b| b.broken).count()
    }
}

/// Create a new peridynamic model.
pub fn new_peridynamics(horizon: f64, stiffness: f64, critical_stretch: f64) -> Peridynamics {
    Peridynamics::new(horizon, stiffness, critical_stretch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_point() {
        let mut pd = Peridynamics::new(1.0, 100.0, 0.5);
        let idx = pd.add_point(0.0, 0.0, 1.0);
        assert_eq!(idx, 0); /* first point index */
    }

    #[test]
    fn test_build_bonds_within_horizon() {
        let mut pd = Peridynamics::new(2.0, 100.0, 0.5);
        pd.add_point(0.0, 0.0, 1.0);
        pd.add_point(1.0, 0.0, 1.0);
        pd.build_bonds();
        assert_eq!(pd.bonds.len(), 1); /* one bond within horizon */
    }

    #[test]
    fn test_build_bonds_outside_horizon() {
        let mut pd = Peridynamics::new(0.5, 100.0, 0.5);
        pd.add_point(0.0, 0.0, 1.0);
        pd.add_point(1.0, 0.0, 1.0);
        pd.build_bonds();
        assert_eq!(pd.bonds.len(), 0); /* too far apart */
    }

    #[test]
    fn test_apply_forces_no_change_at_rest() {
        let mut pd = Peridynamics::new(2.0, 100.0, 0.5);
        pd.add_point(0.0, 0.0, 1.0);
        pd.add_point(1.0, 0.0, 1.0);
        pd.build_bonds();
        pd.apply_forces();
        /* at rest length, no stretch, no force */
        assert!(pd.points[0].vx.abs() < 1e-10); /* no velocity change */
    }

    #[test]
    fn test_bond_breaking() {
        let mut pd = Peridynamics::new(2.0, 100.0, 0.1);
        pd.add_point(0.0, 0.0, 1.0);
        pd.add_point(1.0, 0.0, 1.0);
        pd.build_bonds();
        /* move point far away to exceed critical stretch */
        pd.points[1].x = 5.0;
        pd.apply_forces();
        assert_eq!(pd.broken_bond_count(), 1); /* bond broken */
    }

    #[test]
    fn test_active_bond_count() {
        let mut pd = Peridynamics::new(2.0, 100.0, 0.5);
        pd.add_point(0.0, 0.0, 1.0);
        pd.add_point(1.0, 0.0, 1.0);
        pd.build_bonds();
        assert_eq!(pd.active_bond_count(), 1); /* one active bond */
    }

    #[test]
    fn test_integrate() {
        let mut pd = Peridynamics::new(2.0, 100.0, 0.5);
        pd.add_point(0.0, 0.0, 1.0);
        pd.points[0].vx = 1.0;
        pd.integrate(0.1);
        assert!((pd.points[0].x - 0.1).abs() < 1e-10); /* position updated */
    }

    #[test]
    fn test_new_helper() {
        let pd = new_peridynamics(1.0, 50.0, 0.3);
        assert_eq!(pd.points.len(), 0); /* empty model */
    }

    #[test]
    fn test_three_points() {
        let mut pd = Peridynamics::new(1.5, 100.0, 0.5);
        pd.add_point(0.0, 0.0, 1.0);
        pd.add_point(1.0, 0.0, 1.0);
        pd.add_point(2.0, 0.0, 1.0);
        pd.build_bonds();
        assert_eq!(pd.bonds.len(), 2); /* (0,1) and (1,2) */
    }
}
