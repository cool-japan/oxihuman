// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Body temperature sensor model.

/// Temperature sensor type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemperatureSensorType {
    /// Infrared thermometer.
    Infrared,
    /// Thermocouple.
    Thermocouple,
    /// Resistance Temperature Detector.
    Rtd,
    /// Thermistor (NTC/PTC).
    Thermistor,
}

/// Temperature sensor configuration.
#[derive(Debug, Clone)]
pub struct TemperatureConfig {
    pub sensor_type: TemperatureSensorType,
    /// Measurement range [min, max] in °C.
    pub range_c: [f32; 2],
    /// Accuracy in °C.
    pub accuracy_c: f32,
    /// Resolution in °C.
    pub resolution_c: f32,
    /// Sample rate in Hz.
    pub sample_rate_hz: f32,
}

impl Default for TemperatureConfig {
    fn default() -> Self {
        TemperatureConfig {
            sensor_type: TemperatureSensorType::Infrared,
            range_c: [20.0, 45.0],
            accuracy_c: 0.3,
            resolution_c: 0.1,
            sample_rate_hz: 10.0,
        }
    }
}

/// A temperature measurement.
#[derive(Debug, Clone, PartialEq)]
pub struct TemperatureSample {
    pub time: f32,
    /// Temperature in degrees Celsius.
    pub temp_c: f32,
    /// Body site label (e.g. "forehead", "axilla").
    pub site: String,
}

/// Temperature sensor.
#[derive(Debug)]
pub struct TemperatureSensor {
    pub config: TemperatureConfig,
    samples: Vec<TemperatureSample>,
}

impl TemperatureSensor {
    /// Create a new temperature sensor.
    pub fn new(config: TemperatureConfig) -> Self {
        TemperatureSensor {
            config,
            samples: vec![],
        }
    }

    /// Record a sample.
    pub fn push_sample(&mut self, s: TemperatureSample) {
        self.samples.push(s);
    }

    /// Return sample count.
    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }

    /// Return the latest sample.
    pub fn latest(&self) -> Option<&TemperatureSample> {
        self.samples.last()
    }

    /// Clear all samples.
    pub fn clear(&mut self) {
        self.samples.clear();
    }
}

/// Convert Celsius to Fahrenheit.
pub fn celsius_to_fahrenheit(c: f32) -> f32 {
    c * 9.0 / 5.0 + 32.0
}

/// Convert Fahrenheit to Celsius.
pub fn fahrenheit_to_celsius(f: f32) -> f32 {
    (f - 32.0) * 5.0 / 9.0
}

/// Convert Celsius to Kelvin.
pub fn celsius_to_kelvin(c: f32) -> f32 {
    c + 273.15
}

/// Return `true` if the temperature is within the sensor's measurable range.
pub fn temperature_in_range(temp_c: f32, config: &TemperatureConfig) -> bool {
    (config.range_c[0]..=config.range_c[1]).contains(&temp_c)
}

/// Return `true` if the temperature indicates a fever (≥ 37.5 °C).
pub fn is_fever(temp_c: f32) -> bool {
    temp_c >= 37.5
}

/// Compute the mean temperature from a list of samples.
pub fn mean_temperature(samples: &[TemperatureSample]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    samples.iter().map(|s| s.temp_c).sum::<f32>() / samples.len() as f32
}

/// Find samples from a specific body site.
pub fn samples_at_site<'a>(
    samples: &'a [TemperatureSample],
    site: &str,
) -> Vec<&'a TemperatureSample> {
    samples.iter().filter(|s| s.site == site).collect()
}

/// Quantise a temperature to the sensor resolution.
pub fn quantise(temp_c: f32, resolution_c: f32) -> f32 {
    (temp_c / resolution_c).round() * resolution_c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_celsius_to_fahrenheit() {
        /* 0 °C = 32 °F */
        assert!((celsius_to_fahrenheit(0.0) - 32.0).abs() < 1e-5);
    }

    #[test]
    fn test_fahrenheit_to_celsius() {
        /* 32 °F = 0 °C */
        assert!((fahrenheit_to_celsius(32.0)).abs() < 1e-5);
    }

    #[test]
    fn test_celsius_to_kelvin() {
        /* 0 °C = 273.15 K */
        assert!((celsius_to_kelvin(0.0) - 273.15).abs() < 1e-4);
    }

    #[test]
    fn test_temperature_in_range() {
        /* 36.5 °C is in normal body temperature range */
        assert!(temperature_in_range(36.5, &TemperatureConfig::default()));
    }

    #[test]
    fn test_temperature_out_of_range() {
        /* 50 °C is above sensor max */
        assert!(!temperature_in_range(50.0, &TemperatureConfig::default()));
    }

    #[test]
    fn test_is_fever_true() {
        /* 38.0 °C is a fever */
        assert!(is_fever(38.0));
    }

    #[test]
    fn test_is_fever_false() {
        /* 36.0 °C is not a fever */
        assert!(!is_fever(36.0));
    }

    #[test]
    fn test_mean_temperature() {
        /* mean of 36 and 38 is 37 */
        let samples = vec![
            TemperatureSample {
                time: 0.0,
                temp_c: 36.0,
                site: "forehead".to_string(),
            },
            TemperatureSample {
                time: 1.0,
                temp_c: 38.0,
                site: "axilla".to_string(),
            },
        ];
        assert!((mean_temperature(&samples) - 37.0).abs() < 1e-5);
    }

    #[test]
    fn test_samples_at_site() {
        /* filter by site label */
        let samples = vec![
            TemperatureSample {
                time: 0.0,
                temp_c: 36.5,
                site: "forehead".to_string(),
            },
            TemperatureSample {
                time: 1.0,
                temp_c: 37.0,
                site: "axilla".to_string(),
            },
        ];
        assert_eq!(samples_at_site(&samples, "forehead").len(), 1);
    }

    #[test]
    fn test_quantise() {
        /* quantise to 0.1 resolution */
        assert!((quantise(36.72, 0.1) - 36.7).abs() < 0.001);
    }

    #[test]
    fn test_push_and_count() {
        /* push increments count */
        let mut s = TemperatureSensor::new(TemperatureConfig::default());
        s.push_sample(TemperatureSample {
            time: 0.0,
            temp_c: 36.5,
            site: "forehead".to_string(),
        });
        assert_eq!(s.sample_count(), 1);
    }
}
