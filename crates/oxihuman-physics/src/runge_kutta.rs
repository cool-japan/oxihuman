// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Generic RK4 integrator for scalar and vector ODEs.

#![allow(dead_code)]

/// RK4 step for a scalar ODE dy/dt = f(t, y).
#[allow(dead_code)]
pub fn rk4_scalar<F: Fn(f32, f32) -> f32>(f: &F, t: f32, y: f32, dt: f32) -> f32 {
    let k1 = f(t, y);
    let k2 = f(t + 0.5 * dt, y + 0.5 * dt * k1);
    let k3 = f(t + 0.5 * dt, y + 0.5 * dt * k2);
    let k4 = f(t + dt, y + dt * k3);
    y + dt / 6.0 * (k1 + 2.0 * k2 + 2.0 * k3 + k4)
}

/// RK4 step for a 2D vector ODE d(state)/dt = f(t, state).
#[allow(dead_code)]
pub fn rk4_vec2<F: Fn(f32, [f32; 2]) -> [f32; 2]>(f: &F, t: f32, y: [f32; 2], dt: f32) -> [f32; 2] {
    let k1 = f(t, y);
    let k2 = f(t + 0.5 * dt, add2(y, scale2(k1, 0.5 * dt)));
    let k3 = f(t + 0.5 * dt, add2(y, scale2(k2, 0.5 * dt)));
    let k4 = f(t + dt, add2(y, scale2(k3, dt)));
    add2(
        y,
        scale2(
            add2(add2(k1, scale2(k2, 2.0)), add2(scale2(k3, 2.0), k4)),
            dt / 6.0,
        ),
    )
}

/// RK4 step for a 3D vector ODE.
#[allow(dead_code)]
pub fn rk4_vec3<F: Fn(f32, [f32; 3]) -> [f32; 3]>(f: &F, t: f32, y: [f32; 3], dt: f32) -> [f32; 3] {
    let k1 = f(t, y);
    let k2 = f(t + 0.5 * dt, add3(y, scale3(k1, 0.5 * dt)));
    let k3 = f(t + 0.5 * dt, add3(y, scale3(k2, 0.5 * dt)));
    let k4 = f(t + dt, add3(y, scale3(k3, dt)));
    add3(
        y,
        scale3(
            add3(add3(k1, scale3(k2, 2.0)), add3(scale3(k3, 2.0), k4)),
            dt / 6.0,
        ),
    )
}

/// RK4 step for a generic N-dimensional vector ODE (`Vec<f32>`).
#[allow(dead_code)]
pub fn rk4_vecn<F: Fn(f32, &[f32]) -> Vec<f32>>(f: &F, t: f32, y: &[f32], dt: f32) -> Vec<f32> {
    let k1 = f(t, y);
    let y2: Vec<f32> = y.iter().zip(&k1).map(|(a, b)| a + 0.5 * dt * b).collect();
    let k2 = f(t + 0.5 * dt, &y2);
    let y3: Vec<f32> = y.iter().zip(&k2).map(|(a, b)| a + 0.5 * dt * b).collect();
    let k3 = f(t + 0.5 * dt, &y3);
    let y4: Vec<f32> = y.iter().zip(&k3).map(|(a, b)| a + dt * b).collect();
    let k4 = f(t + dt, &y4);
    y.iter()
        .zip(k1.iter().zip(k2.iter().zip(k3.iter().zip(k4.iter()))))
        .map(|(yi, (k1i, (k2i, (k3i, k4i))))| yi + dt / 6.0 * (k1i + 2.0 * k2i + 2.0 * k3i + k4i))
        .collect()
}

fn add2(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    [a[0] + b[0], a[1] + b[1]]
}
fn scale2(a: [f32; 2], s: f32) -> [f32; 2] {
    [a[0] * s, a[1] * s]
}
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn scale3(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

/// Euler step for scalar ODE (for comparison).
#[allow(dead_code)]
pub fn euler_scalar<F: Fn(f32, f32) -> f32>(f: &F, t: f32, y: f32, dt: f32) -> f32 {
    y + dt * f(t, y)
}

/// Integrate scalar ODE for `steps` steps, returning trajectory.
#[allow(dead_code)]
pub fn integrate_scalar<F: Fn(f32, f32) -> f32>(
    f: &F,
    t0: f32,
    y0: f32,
    dt: f32,
    steps: usize,
) -> Vec<(f32, f32)> {
    let mut t = t0;
    let mut y = y0;
    let mut traj = Vec::with_capacity(steps + 1);
    traj.push((t, y));
    for _ in 0..steps {
        y = rk4_scalar(f, t, y, dt);
        t += dt;
        traj.push((t, y));
    }
    traj
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rk4_scalar_exponential() {
        // dy/dt = -y → y = y0 * e^(-t)
        let f = |_t: f32, y: f32| -y;
        let y0 = 1.0f32;
        let dt = 0.1;
        let mut y = y0;
        let mut t = 0.0;
        for _ in 0..10 {
            y = rk4_scalar(&f, t, y, dt);
            t += dt;
        }
        let expected = (-1.0f32).exp();
        assert!((y - expected).abs() < 0.001, "y={y}, expected={expected}");
    }

    #[test]
    fn rk4_scalar_zero_derivative() {
        let f = |_t: f32, _y: f32| 0.0f32;
        let y = rk4_scalar(&f, 0.0, 5.0, 0.1);
        assert!((y - 5.0).abs() < 1e-5);
    }

    #[test]
    fn rk4_vec2_constant_velocity() {
        let f = |_t: f32, _y: [f32; 2]| [1.0f32, 0.0f32];
        let y = rk4_vec2(&f, 0.0, [0.0, 0.0], 1.0);
        assert!((y[0] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn rk4_vec3_zero_deriv() {
        let f = |_t: f32, _y: [f32; 3]| [0.0f32, 0.0, 0.0];
        let y = rk4_vec3(&f, 0.0, [1.0, 2.0, 3.0], 0.1);
        assert!((y[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn rk4_vecn_exponential_decay() {
        let f = |_t: f32, y: &[f32]| vec![-y[0]];
        let y0 = vec![1.0f32];
        let mut y = y0;
        let mut t = 0.0;
        for _ in 0..10 {
            y = rk4_vecn(&f, t, &y, 0.1);
            t += 0.1;
        }
        let expected = (-1.0f32).exp();
        assert!((y[0] - expected).abs() < 0.001);
    }

    #[test]
    fn euler_less_accurate_than_rk4() {
        let f = |_t: f32, y: f32| -y;
        let y_rk4 = rk4_scalar(&f, 0.0, 1.0, 0.5);
        let y_euler = euler_scalar(&f, 0.0, 1.0, 0.5);
        let exact = (-0.5f32).exp();
        let err_rk4 = (y_rk4 - exact).abs();
        let err_euler = (y_euler - exact).abs();
        assert!(err_rk4 < err_euler, "rk4={err_rk4} not < euler={err_euler}");
    }

    #[test]
    fn integrate_scalar_length() {
        let f = |_t: f32, y: f32| -y;
        let traj = integrate_scalar(&f, 0.0, 1.0, 0.1, 10);
        assert_eq!(traj.len(), 11);
    }

    #[test]
    fn integrate_scalar_time_advances() {
        let f = |_t: f32, y: f32| -y;
        let traj = integrate_scalar(&f, 0.0, 1.0, 0.1, 5);
        assert!((traj[5].0 - 0.5).abs() < 1e-4);
    }

    #[test]
    fn rk4_harmonic_oscillator() {
        // d[x,v]/dt = [v, -x]
        let f = |_t: f32, y: [f32; 2]| [y[1], -y[0]];
        let mut y = [1.0f32, 0.0f32];
        let mut t = 0.0;
        for _ in 0..100 {
            y = rk4_vec2(&f, t, y, 0.01);
            t += 0.01;
        }
        // Energy x²+v² should be conserved ≈ 1.0
        let e = y[0] * y[0] + y[1] * y[1];
        assert!((e - 1.0).abs() < 0.01, "energy={e}");
    }

    #[test]
    fn add2_correct() {
        let r = add2([1.0, 2.0], [3.0, 4.0]);
        assert!((r[0] - 4.0).abs() < 1e-5 && (r[1] - 6.0).abs() < 1e-5);
    }
}
