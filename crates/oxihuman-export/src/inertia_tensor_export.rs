// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct InertiaTensor {
    pub mass: f32,
    pub tensor: [[f32; 3]; 3],
    pub center_of_mass: [f32; 3],
}

pub fn new_inertia_tensor(mass: f32, com: [f32; 3]) -> InertiaTensor {
    InertiaTensor {
        mass,
        tensor: [[0.0; 3]; 3],
        center_of_mass: com,
    }
}

pub fn inertia_set_diagonal(t: &mut InertiaTensor, ixx: f32, iyy: f32, izz: f32) {
    t.tensor[0][0] = ixx;
    t.tensor[1][1] = iyy;
    t.tensor[2][2] = izz;
}

pub fn inertia_principal_moments(t: &InertiaTensor) -> [f32; 3] {
    [t.tensor[0][0], t.tensor[1][1], t.tensor[2][2]]
}

pub fn inertia_is_symmetric(t: &InertiaTensor) -> bool {
    for i in 0..3 {
        for j in 0..3 {
            if (t.tensor[i][j] - t.tensor[j][i]).abs() > 1e-6 {
                return false;
            }
        }
    }
    true
}

pub fn inertia_to_json(t: &InertiaTensor) -> String {
    let pm = inertia_principal_moments(t);
    format!(
        "{{\"mass\":{:.4},\"ixx\":{:.4},\"iyy\":{:.4},\"izz\":{:.4}}}",
        t.mass, pm[0], pm[1], pm[2]
    )
}

pub fn inertia_to_bytes(t: &InertiaTensor) -> Vec<u8> {
    let mut b = Vec::new();
    b.extend_from_slice(&t.mass.to_le_bytes());
    for row in &t.tensor {
        for &v in row {
            b.extend_from_slice(&v.to_le_bytes());
        }
    }
    b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_inertia_tensor() {
        /* mass stored correctly */
        let t = new_inertia_tensor(70.0, [0.0; 3]);
        assert!((t.mass - 70.0).abs() < 1e-5);
    }

    #[test]
    fn test_inertia_set_diagonal() {
        /* diagonal elements set */
        let mut t = new_inertia_tensor(1.0, [0.0; 3]);
        inertia_set_diagonal(&mut t, 1.0, 2.0, 3.0);
        assert!((t.tensor[1][1] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_inertia_principal_moments() {
        /* returns diagonal */
        let mut t = new_inertia_tensor(1.0, [0.0; 3]);
        inertia_set_diagonal(&mut t, 4.0, 5.0, 6.0);
        let pm = inertia_principal_moments(&t);
        assert!((pm[2] - 6.0).abs() < 1e-6);
    }

    #[test]
    fn test_inertia_is_symmetric_diagonal() {
        /* diagonal matrix is symmetric */
        let mut t = new_inertia_tensor(1.0, [0.0; 3]);
        inertia_set_diagonal(&mut t, 1.0, 2.0, 3.0);
        assert!(inertia_is_symmetric(&t));
    }

    #[test]
    fn test_inertia_to_json() {
        /* json contains mass */
        let t = new_inertia_tensor(5.0, [0.0; 3]);
        let j = inertia_to_json(&t);
        assert!(j.contains("mass"));
    }

    #[test]
    fn test_inertia_to_bytes() {
        /* bytes have correct length: 4 (mass) + 36 (3x3 tensor) = 40 */
        let t = new_inertia_tensor(1.0, [0.0; 3]);
        let b = inertia_to_bytes(&t);
        assert_eq!(b.len(), 40);
    }
}
