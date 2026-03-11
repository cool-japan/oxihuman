// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Pressure mat grid sensor model.

/// Pressure mat configuration.
#[derive(Debug, Clone)]
pub struct PressureMatConfig {
    /// Number of sensor rows.
    pub rows: usize,
    /// Number of sensor columns.
    pub cols: usize,
    /// Physical size of each cell in metres [width, height].
    pub cell_size_m: [f32; 2],
    /// Maximum pressure per cell in kPa.
    pub max_pressure_kpa: f32,
    /// Sample rate in Hz.
    pub sample_rate_hz: f32,
}

impl Default for PressureMatConfig {
    fn default() -> Self {
        PressureMatConfig {
            rows: 64,
            cols: 48,
            cell_size_m: [0.005, 0.005],
            max_pressure_kpa: 300.0,
            sample_rate_hz: 100.0,
        }
    }
}

/// A pressure mat frame (one pressure value per cell, row-major).
#[derive(Debug, Clone)]
pub struct PressureFrame {
    pub time: f32,
    /// Pressure values in kPa, length = rows × cols.
    pub cells: Vec<f32>,
}

/// Pressure mat sensor.
#[derive(Debug)]
pub struct PressureMatSensor {
    pub config: PressureMatConfig,
    frames: Vec<PressureFrame>,
}

impl PressureMatSensor {
    /// Create a new pressure mat sensor.
    pub fn new(config: PressureMatConfig) -> Self {
        PressureMatSensor {
            config,
            frames: vec![],
        }
    }

    /// Record a frame.
    pub fn push_frame(&mut self, frame: PressureFrame) {
        self.frames.push(frame);
    }

    /// Return the number of frames.
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Return the latest frame.
    pub fn latest(&self) -> Option<&PressureFrame> {
        self.frames.last()
    }

    /// Clear all frames.
    pub fn clear(&mut self) {
        self.frames.clear();
    }

    /// Return the expected cell count.
    pub fn cell_count(&self) -> usize {
        self.config.rows * self.config.cols
    }
}

/// Compute the total force on the mat from a frame (cells × cell area × pressure).
pub fn total_force_n(frame: &PressureFrame, config: &PressureMatConfig) -> f32 {
    let cell_area = config.cell_size_m[0] * config.cell_size_m[1];
    let total_kpa: f32 = frame.cells.iter().sum();
    total_kpa * 1000.0 * cell_area /* kPa → Pa, then × area → N */
}

/// Compute the centre of pressure from a pressure frame.
pub fn centre_of_pressure(frame: &PressureFrame, config: &PressureMatConfig) -> Option<[f32; 2]> {
    let total: f32 = frame.cells.iter().sum();
    if total < 1e-9 {
        return None;
    }
    let mut wx = 0.0f32;
    let mut wy = 0.0f32;
    for r in 0..config.rows {
        for c in 0..config.cols {
            let idx = r * config.cols + c;
            if idx >= frame.cells.len() {
                break;
            }
            let p = frame.cells[idx];
            wx += p * c as f32 * config.cell_size_m[0];
            wy += p * r as f32 * config.cell_size_m[1];
        }
    }
    Some([wx / total, wy / total])
}

/// Count cells above a pressure threshold.
pub fn active_cells(frame: &PressureFrame, threshold_kpa: f32) -> usize {
    frame.cells.iter().filter(|&&p| p > threshold_kpa).count()
}

/// Return the peak pressure in a frame.
pub fn peak_pressure(frame: &PressureFrame) -> f32 {
    frame.cells.iter().cloned().fold(0.0f32, f32::max)
}

/// Compute the contact area in m².
pub fn contact_area_m2(
    frame: &PressureFrame,
    config: &PressureMatConfig,
    threshold_kpa: f32,
) -> f32 {
    let n = active_cells(frame, threshold_kpa) as f32;
    n * config.cell_size_m[0] * config.cell_size_m[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_frame(rows: usize, cols: usize, val: f32) -> PressureFrame {
        PressureFrame {
            time: 0.0,
            cells: vec![val; rows * cols],
        }
    }

    #[test]
    fn test_cell_count() {
        /* cell count is rows × cols */
        let s = PressureMatSensor::new(PressureMatConfig::default());
        assert_eq!(s.cell_count(), 64 * 48);
    }

    #[test]
    fn test_push_frame() {
        /* push increments count */
        let mut s = PressureMatSensor::new(PressureMatConfig::default());
        s.push_frame(make_frame(2, 2, 0.0));
        assert_eq!(s.frame_count(), 1);
    }

    #[test]
    fn test_peak_pressure() {
        /* peak finds max */
        let f = PressureFrame {
            time: 0.0,
            cells: vec![1.0, 5.0, 3.0],
        };
        assert!((peak_pressure(&f) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_active_cells() {
        /* count cells above threshold */
        let f = PressureFrame {
            time: 0.0,
            cells: vec![0.0, 10.0, 20.0],
        };
        assert_eq!(active_cells(&f, 5.0), 2);
    }

    #[test]
    fn test_cop_uniform() {
        /* CoP of uniform field is at the centre */
        let cfg = PressureMatConfig {
            rows: 2,
            cols: 2,
            cell_size_m: [1.0, 1.0],
            max_pressure_kpa: 100.0,
            sample_rate_hz: 1.0,
        };
        let f = make_frame(2, 2, 1.0);
        let cop = centre_of_pressure(&f, &cfg).unwrap();
        assert!((cop[0] - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_cop_zero_pressure() {
        /* zero pressure returns None */
        let cfg = PressureMatConfig::default();
        let f = make_frame(2, 2, 0.0);
        assert!(centre_of_pressure(&f, &cfg).is_none());
    }

    #[test]
    fn test_contact_area() {
        /* contact area is active cells × cell area */
        let cfg = PressureMatConfig {
            rows: 1,
            cols: 4,
            cell_size_m: [0.01, 0.01],
            max_pressure_kpa: 100.0,
            sample_rate_hz: 1.0,
        };
        let f = PressureFrame {
            time: 0.0,
            cells: vec![0.0, 10.0, 10.0, 0.0],
        };
        let area = contact_area_m2(&f, &cfg, 5.0);
        assert!((area - 0.0002).abs() < 1e-9);
    }

    #[test]
    fn test_total_force() {
        /* total force is non-zero for non-zero pressure */
        let cfg = PressureMatConfig {
            rows: 1,
            cols: 1,
            cell_size_m: [0.01, 0.01],
            max_pressure_kpa: 100.0,
            sample_rate_hz: 1.0,
        };
        let f = PressureFrame {
            time: 0.0,
            cells: vec![100.0],
        }; /* 100 kPa */
        let force = total_force_n(&f, &cfg);
        assert!(force > 0.0);
    }

    #[test]
    fn test_clear() {
        /* clear removes frames */
        let mut s = PressureMatSensor::new(PressureMatConfig::default());
        s.push_frame(make_frame(1, 1, 0.0));
        s.clear();
        assert_eq!(s.frame_count(), 0);
    }

    #[test]
    fn test_latest() {
        /* latest returns last frame */
        let mut s = PressureMatSensor::new(PressureMatConfig::default());
        s.push_frame(PressureFrame {
            time: 1.5,
            cells: vec![0.0],
        });
        assert_eq!(s.latest().unwrap().time, 1.5);
    }
}
