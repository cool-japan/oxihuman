// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Integrates accumulated forces into velocity and position changes.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ForceIntegrator {
    forces: Vec<[f32; 3]>,
    method: IntegrationMethod,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IntegrationMethod {
    Euler,
    SemiImplicitEuler,
    Verlet,
}

#[allow(dead_code)]
impl ForceIntegrator {
    pub fn new(method: IntegrationMethod) -> Self {
        Self {
            forces: Vec::new(),
            method,
        }
    }

    pub fn euler() -> Self {
        Self::new(IntegrationMethod::Euler)
    }

    pub fn add_force(&mut self, force: [f32; 3]) {
        self.forces.push(force);
    }

    pub fn total_force(&self) -> [f32; 3] {
        let mut total = [0.0f32; 3];
        for f in &self.forces {
            for i in 0..3 {
                total[i] += f[i];
            }
        }
        total
    }

    pub fn acceleration(&self, inv_mass: f32) -> [f32; 3] {
        let f = self.total_force();
        [f[0] * inv_mass, f[1] * inv_mass, f[2] * inv_mass]
    }

    pub fn integrate_velocity(
        &self,
        velocity: [f32; 3],
        inv_mass: f32,
        dt: f32,
    ) -> [f32; 3] {
        let acc = self.acceleration(inv_mass);
        [
            velocity[0] + acc[0] * dt,
            velocity[1] + acc[1] * dt,
            velocity[2] + acc[2] * dt,
        ]
    }

    pub fn integrate_position(
        &self,
        position: [f32; 3],
        velocity: [f32; 3],
        inv_mass: f32,
        dt: f32,
    ) -> ([f32; 3], [f32; 3]) {
        match self.method {
            IntegrationMethod::Euler => {
                let new_pos = [
                    position[0] + velocity[0] * dt,
                    position[1] + velocity[1] * dt,
                    position[2] + velocity[2] * dt,
                ];
                let new_vel = self.integrate_velocity(velocity, inv_mass, dt);
                (new_pos, new_vel)
            }
            IntegrationMethod::SemiImplicitEuler => {
                let new_vel = self.integrate_velocity(velocity, inv_mass, dt);
                let new_pos = [
                    position[0] + new_vel[0] * dt,
                    position[1] + new_vel[1] * dt,
                    position[2] + new_vel[2] * dt,
                ];
                (new_pos, new_vel)
            }
            IntegrationMethod::Verlet => {
                let acc = self.acceleration(inv_mass);
                let new_pos = [
                    position[0] + velocity[0] * dt + 0.5 * acc[0] * dt * dt,
                    position[1] + velocity[1] * dt + 0.5 * acc[1] * dt * dt,
                    position[2] + velocity[2] * dt + 0.5 * acc[2] * dt * dt,
                ];
                let new_vel = self.integrate_velocity(velocity, inv_mass, dt);
                (new_pos, new_vel)
            }
        }
    }

    pub fn clear(&mut self) {
        self.forces.clear();
    }

    pub fn num_forces(&self) -> usize {
        self.forces.len()
    }

    pub fn method(&self) -> IntegrationMethod {
        self.method
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let fi = ForceIntegrator::euler();
        assert_eq!(fi.method(), IntegrationMethod::Euler);
        assert_eq!(fi.num_forces(), 0);
    }

    #[test]
    fn test_total_force() {
        let mut fi = ForceIntegrator::euler();
        fi.add_force([1.0, 0.0, 0.0]);
        fi.add_force([0.0, 2.0, 0.0]);
        let total = fi.total_force();
        assert!((total[0] - 1.0).abs() < 1e-9);
        assert!((total[1] - 2.0).abs() < 1e-9);
    }

    #[test]
    fn test_acceleration() {
        let mut fi = ForceIntegrator::euler();
        fi.add_force([10.0, 0.0, 0.0]);
        let acc = fi.acceleration(0.5); // inv_mass = 0.5 means mass = 2
        assert!((acc[0] - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_integrate_velocity() {
        let mut fi = ForceIntegrator::euler();
        fi.add_force([0.0, -9.81, 0.0]);
        let new_vel = fi.integrate_velocity([0.0; 3], 1.0, 1.0);
        assert!((new_vel[1] - (-9.81)).abs() < 1e-5);
    }

    #[test]
    fn test_euler_position() {
        let fi = ForceIntegrator::euler();
        let (pos, _) = fi.integrate_position([0.0; 3], [1.0, 0.0, 0.0], 1.0, 1.0);
        assert!((pos[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_semi_implicit_euler() {
        let mut fi = ForceIntegrator::new(IntegrationMethod::SemiImplicitEuler);
        fi.add_force([10.0, 0.0, 0.0]);
        let (pos, vel) = fi.integrate_position([0.0; 3], [0.0; 3], 1.0, 1.0);
        assert!((vel[0] - 10.0).abs() < 1e-5);
        assert!((pos[0] - 10.0).abs() < 1e-5);
    }

    #[test]
    fn test_verlet() {
        let fi = ForceIntegrator::new(IntegrationMethod::Verlet);
        let (pos, _) = fi.integrate_position([0.0; 3], [5.0, 0.0, 0.0], 1.0, 1.0);
        assert!((pos[0] - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_clear() {
        let mut fi = ForceIntegrator::euler();
        fi.add_force([1.0; 3]);
        fi.clear();
        assert_eq!(fi.num_forces(), 0);
        let total = fi.total_force();
        assert_eq!(total, [0.0; 3]);
    }

    #[test]
    fn test_no_force_no_change() {
        let fi = ForceIntegrator::euler();
        let vel = fi.integrate_velocity([5.0, 0.0, 0.0], 1.0, 1.0);
        assert!((vel[0] - 5.0).abs() < 1e-9);
    }

    #[test]
    fn test_multiple_forces() {
        let mut fi = ForceIntegrator::euler();
        fi.add_force([1.0, 0.0, 0.0]);
        fi.add_force([1.0, 0.0, 0.0]);
        fi.add_force([1.0, 0.0, 0.0]);
        assert_eq!(fi.num_forces(), 3);
        let total = fi.total_force();
        assert!((total[0] - 3.0).abs() < 1e-9);
    }
}
