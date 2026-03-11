// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Acoustic impedance mismatch and reflection coefficients.

/// Acoustic impedance of a medium Z = ρ * c (Pa·s/m = Rayl).
pub fn acoustic_impedance(density: f32, speed_of_sound: f32) -> f32 {
    density * speed_of_sound
}

/// Pressure reflection coefficient at a boundary between two media.
/// R = (Z2 - Z1) / (Z2 + Z1)
pub fn pressure_reflection_coeff(z1: f32, z2: f32) -> f32 {
    (z2 - z1) / (z2 + z1 + 1e-15)
}

/// Pressure transmission coefficient.
/// T = 2Z2 / (Z2 + Z1)
pub fn pressure_transmission_coeff(z1: f32, z2: f32) -> f32 {
    2.0 * z2 / (z2 + z1 + 1e-15)
}

/// Intensity reflection coefficient.
/// R_I = R^2
pub fn intensity_reflection_coeff(z1: f32, z2: f32) -> f32 {
    pressure_reflection_coeff(z1, z2).powi(2)
}

/// Intensity transmission coefficient.
/// T_I = 1 - R_I
pub fn intensity_transmission_coeff(z1: f32, z2: f32) -> f32 {
    1.0 - intensity_reflection_coeff(z1, z2)
}

/// Acoustic standing wave ratio (SWR).
pub fn standing_wave_ratio(z1: f32, z2: f32) -> f32 {
    let r = pressure_reflection_coeff(z1, z2).abs();
    (1.0 + r) / (1.0 - r + 1e-15)
}

/// Transmission loss in dB due to impedance mismatch.
pub fn transmission_loss_db(z1: f32, z2: f32) -> f32 {
    let t_i = intensity_transmission_coeff(z1, z2).max(1e-12);
    -10.0 * t_i.log10()
}

/// An acoustic medium.
#[derive(Debug, Clone)]
pub struct AcousticMedium {
    pub density: f32,
    pub speed: f32,
    pub name: String,
}

impl AcousticMedium {
    /// Acoustic impedance.
    pub fn impedance(&self) -> f32 {
        acoustic_impedance(self.density, self.speed)
    }
}

/// Construct a new AcousticMedium.
pub fn new_acoustic_medium(name: &str, density: f32, speed: f32) -> AcousticMedium {
    AcousticMedium {
        density,
        speed,
        name: name.to_string(),
    }
}

/// Common media presets.
pub fn air_medium() -> AcousticMedium {
    new_acoustic_medium("air", 1.293, 343.0)
}

pub fn water_medium() -> AcousticMedium {
    new_acoustic_medium("water", 1000.0, 1480.0)
}

pub fn steel_medium() -> AcousticMedium {
    new_acoustic_medium("steel", 7800.0, 5100.0)
}

/// Compute reflection and transmission for an interface.
#[derive(Debug, Clone)]
pub struct InterfaceResult {
    pub r_pressure: f32,
    pub t_pressure: f32,
    pub r_intensity: f32,
    pub t_intensity: f32,
}

/// Analyze an acoustic interface between `medium1` and `medium2`.
pub fn analyze_interface(m1: &AcousticMedium, m2: &AcousticMedium) -> InterfaceResult {
    let z1 = m1.impedance();
    let z2 = m2.impedance();
    InterfaceResult {
        r_pressure: pressure_reflection_coeff(z1, z2),
        t_pressure: pressure_transmission_coeff(z1, z2),
        r_intensity: intensity_reflection_coeff(z1, z2),
        t_intensity: intensity_transmission_coeff(z1, z2),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_impedance_air() {
        /* air impedance is ~444 Rayl */
        let z = acoustic_impedance(1.293, 343.0);
        assert!((z - 443.5).abs() < 1.0, "z={z}");
    }

    #[test]
    fn test_reflection_coeff_same_medium() {
        /* same medium: reflection = 0 */
        let r = pressure_reflection_coeff(400.0, 400.0);
        assert!(r.abs() < 1e-6, "r={r}");
    }

    #[test]
    fn test_transmission_coeff_same_medium() {
        /* same medium: transmission = 1 */
        let t = pressure_transmission_coeff(400.0, 400.0);
        assert!((t - 1.0).abs() < 1e-6, "t={t}");
    }

    #[test]
    fn test_energy_conservation() {
        /* R_I + T_I = 1 */
        let z1 = 400.0f32;
        let z2 = 1_500_000.0f32;
        let r = intensity_reflection_coeff(z1, z2);
        let t = intensity_transmission_coeff(z1, z2);
        assert!((r + t - 1.0).abs() < 1e-5, "r+t={}", r + t);
    }

    #[test]
    fn test_air_water_high_reflection() {
        /* air-water interface has high reflection (>99%) */
        let air = air_medium();
        let water = water_medium();
        let res = analyze_interface(&air, &water);
        assert!(res.r_intensity > 0.99, "r_i={}", res.r_intensity);
    }

    #[test]
    fn test_transmission_loss_positive() {
        /* transmission loss is positive for mismatch */
        let loss = transmission_loss_db(400.0, 1_500_000.0);
        assert!(loss > 0.0, "loss={loss}");
    }

    #[test]
    fn test_swr_matched() {
        /* SWR = 1.0 for matched impedances */
        let swr = standing_wave_ratio(100.0, 100.0);
        assert!(swr < 1.01, "swr={swr}");
    }

    #[test]
    fn test_medium_impedance() {
        /* AcousticMedium.impedance() = density * speed */
        let m = new_acoustic_medium("test", 2.0, 500.0);
        assert!((m.impedance() - 1000.0).abs() < 1e-3);
    }

    #[test]
    fn test_water_medium() {
        /* water medium has density 1000 and speed ~1480 */
        let w = water_medium();
        assert!((w.density - 1000.0).abs() < 1.0);
    }
}
