// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

/// Mass properties for a rigid body.
/// `inertia` is stored as the 6 unique components of the symmetric inertia tensor:
/// [Ixx, Iyy, Izz, Ixy, Ixz, Iyz].
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct MassProps {
    pub mass: f32,
    pub center_of_mass: [f32; 3],
    pub inertia: [f32; 6],
}

/// Create default mass properties (unit mass, zero COM, identity inertia diagonal).
#[allow(dead_code)]
pub fn default_mass_props() -> MassProps {
    MassProps {
        mass: 1.0,
        center_of_mass: [0.0; 3],
        inertia: [1.0, 1.0, 1.0, 0.0, 0.0, 0.0],
    }
}

/// Translate the inertia tensor by an offset using the parallel-axis theorem.
/// `inertia` = [Ixx, Iyy, Izz, Ixy, Ixz, Iyz], offset = [dx, dy, dz].
#[allow(dead_code)]
pub fn translate_inertia(inertia: [f32; 6], mass: f32, offset: [f32; 3]) -> [f32; 6] {
    let [dx, dy, dz] = offset;
    let r2 = dx * dx + dy * dy + dz * dz;
    [
        inertia[0] + mass * (r2 - dx * dx),
        inertia[1] + mass * (r2 - dy * dy),
        inertia[2] + mass * (r2 - dz * dz),
        inertia[3] - mass * dx * dy,
        inertia[4] - mass * dx * dz,
        inertia[5] - mass * dy * dz,
    ]
}

/// Combine two mass properties, where `b`'s center is offset by `offset` from `a`'s frame.
#[allow(dead_code)]
pub fn combine_mass_props(a: &MassProps, b: &MassProps, offset: [f32; 3]) -> MassProps {
    let total_mass = a.mass + b.mass;
    if total_mass <= 0.0 {
        return default_mass_props();
    }
    // Combined center of mass
    let com = [
        (a.mass * a.center_of_mass[0] + b.mass * (b.center_of_mass[0] + offset[0])) / total_mass,
        (a.mass * a.center_of_mass[1] + b.mass * (b.center_of_mass[1] + offset[1])) / total_mass,
        (a.mass * a.center_of_mass[2] + b.mass * (b.center_of_mass[2] + offset[2])) / total_mass,
    ];
    // Translate both inertia tensors to the combined COM
    let da = [
        a.center_of_mass[0] - com[0],
        a.center_of_mass[1] - com[1],
        a.center_of_mass[2] - com[2],
    ];
    let db = [
        b.center_of_mass[0] + offset[0] - com[0],
        b.center_of_mass[1] + offset[1] - com[1],
        b.center_of_mass[2] + offset[2] - com[2],
    ];
    let ia = translate_inertia(a.inertia, a.mass, da);
    let ib = translate_inertia(b.inertia, b.mass, db);
    let inertia = [
        ia[0] + ib[0],
        ia[1] + ib[1],
        ia[2] + ib[2],
        ia[3] + ib[3],
        ia[4] + ib[4],
        ia[5] + ib[5],
    ];
    MassProps {
        mass: total_mass,
        center_of_mass: com,
        inertia,
    }
}

/// Mass properties for a uniform sphere.
/// Inertia = (2/5) * m * r^2 for each diagonal.
#[allow(dead_code)]
pub fn mass_props_sphere(mass: f32, radius: f32) -> MassProps {
    let i = 0.4 * mass * radius * radius;
    MassProps {
        mass,
        center_of_mass: [0.0; 3],
        inertia: [i, i, i, 0.0, 0.0, 0.0],
    }
}

/// Mass properties for a uniform box with half-extents `hx`, `hy`, `hz`.
/// Ixx = (1/12)*m*(hy^2 + hz^2), etc.
#[allow(dead_code)]
pub fn mass_props_box(mass: f32, hx: f32, hy: f32, hz: f32) -> MassProps {
    let f = mass / 3.0;
    let ixx = f * (hy * hy + hz * hz);
    let iyy = f * (hx * hx + hz * hz);
    let izz = f * (hx * hx + hy * hy);
    MassProps {
        mass,
        center_of_mass: [0.0; 3],
        inertia: [ixx, iyy, izz, 0.0, 0.0, 0.0],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_mass_is_one() {
        let mp = default_mass_props();
        assert_eq!(mp.mass, 1.0);
    }

    #[test]
    fn sphere_inertia_isotropic() {
        let mp = mass_props_sphere(1.0, 1.0);
        assert!((mp.inertia[0] - mp.inertia[1]).abs() < 1e-6);
        assert!((mp.inertia[1] - mp.inertia[2]).abs() < 1e-6);
    }

    #[test]
    fn sphere_inertia_value() {
        let mp = mass_props_sphere(5.0, 2.0);
        // I = 0.4 * 5 * 4 = 8
        assert!((mp.inertia[0] - 8.0).abs() < 1e-5);
    }

    #[test]
    fn box_inertia_symmetry() {
        // Cube: hx=hy=hz -> Ixx=Iyy=Izz
        let mp = mass_props_box(1.0, 1.0, 1.0, 1.0);
        assert!((mp.inertia[0] - mp.inertia[1]).abs() < 1e-6);
        assert!((mp.inertia[1] - mp.inertia[2]).abs() < 1e-6);
    }

    #[test]
    fn translate_inertia_zero_offset_unchanged() {
        let i = [1.0f32, 2.0, 3.0, 0.0, 0.0, 0.0];
        let result = translate_inertia(i, 1.0, [0.0, 0.0, 0.0]);
        for k in 0..6 {
            assert!((result[k] - i[k]).abs() < 1e-6);
        }
    }

    #[test]
    fn combine_mass_props_total_mass() {
        let a = mass_props_sphere(2.0, 0.5);
        let b = mass_props_sphere(3.0, 0.5);
        let c = combine_mass_props(&a, &b, [1.0, 0.0, 0.0]);
        assert!((c.mass - 5.0).abs() < 1e-5);
    }

    #[test]
    fn combine_com_between_parts() {
        let a = default_mass_props(); // mass=1, com=[0,0,0]
        let b = default_mass_props(); // mass=1, com=[0,0,0]
        let c = combine_mass_props(&a, &b, [2.0, 0.0, 0.0]);
        // Combined COM should be at x=1.0
        assert!((c.center_of_mass[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn translate_inertia_increases_diagonal() {
        let i = [1.0f32, 1.0, 1.0, 0.0, 0.0, 0.0];
        let result = translate_inertia(i, 1.0, [1.0, 0.0, 0.0]);
        // Ixx stays same (parallel to x offset only increases Iyy, Izz)
        assert!(result[1] > 1.0);
        assert!(result[2] > 1.0);
    }

    #[test]
    fn box_mass_preserved() {
        let mp = mass_props_box(10.0, 1.0, 2.0, 3.0);
        assert!((mp.mass - 10.0).abs() < 1e-5);
    }

    #[test]
    fn sphere_off_diagonal_zero() {
        let mp = mass_props_sphere(2.0, 1.0);
        assert_eq!(mp.inertia[3], 0.0);
        assert_eq!(mp.inertia[4], 0.0);
        assert_eq!(mp.inertia[5], 0.0);
    }
}
