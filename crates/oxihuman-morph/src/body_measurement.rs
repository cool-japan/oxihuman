#![allow(dead_code)]

/// A body measurement with name and value.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodyMeasurement {
    points: Vec<MeasurementPoint>,
}

/// A single measurement point.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeasurementPoint {
    name: String,
    value_cm: f32,
}

/// Create a new empty body measurement collection.
#[allow(dead_code)]
pub fn new_body_measurement() -> BodyMeasurement {
    BodyMeasurement { points: Vec::new() }
}

/// Add or compute a chest measurement.
#[allow(dead_code)]
pub fn chest_measurement(bm: &mut BodyMeasurement, value_cm: f32) {
    bm.points.push(MeasurementPoint { name: "chest".to_string(), value_cm });
}

/// Add or compute a hip measurement.
#[allow(dead_code)]
pub fn hip_measurement(bm: &mut BodyMeasurement, value_cm: f32) {
    bm.points.push(MeasurementPoint { name: "hip".to_string(), value_cm });
}

/// Add or compute a waist measurement.
#[allow(dead_code)]
pub fn waist_measurement(bm: &mut BodyMeasurement, value_cm: f32) {
    bm.points.push(MeasurementPoint { name: "waist".to_string(), value_cm });
}

/// Add or compute an inseam measurement.
#[allow(dead_code)]
pub fn inseam_measurement(bm: &mut BodyMeasurement, value_cm: f32) {
    bm.points.push(MeasurementPoint { name: "inseam".to_string(), value_cm });
}

/// Add or compute an arm length measurement.
#[allow(dead_code)]
pub fn arm_length_measurement(bm: &mut BodyMeasurement, value_cm: f32) {
    bm.points.push(MeasurementPoint { name: "arm_length".to_string(), value_cm });
}

/// Serialize measurements to a JSON string.
#[allow(dead_code)]
pub fn measurements_to_json(bm: &BodyMeasurement) -> String {
    let entries: Vec<String> = bm.points.iter()
        .map(|p| format!(r#"{{"name":"{}","value_cm":{:.2}}}"#, p.name, p.value_cm))
        .collect();
    format!("[{}]", entries.join(","))
}

/// Return the total number of measurement points.
#[allow(dead_code)]
pub fn measurement_count(bm: &BodyMeasurement) -> usize {
    bm.points.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_body_measurement() {
        let bm = new_body_measurement();
        assert_eq!(measurement_count(&bm), 0);
    }

    #[test]
    fn test_chest_measurement() {
        let mut bm = new_body_measurement();
        chest_measurement(&mut bm, 95.0);
        assert_eq!(measurement_count(&bm), 1);
        assert_eq!(bm.points[0].name, "chest");
    }

    #[test]
    fn test_hip_measurement() {
        let mut bm = new_body_measurement();
        hip_measurement(&mut bm, 100.0);
        assert_eq!(bm.points[0].value_cm, 100.0);
    }

    #[test]
    fn test_waist_measurement() {
        let mut bm = new_body_measurement();
        waist_measurement(&mut bm, 75.0);
        assert_eq!(bm.points[0].name, "waist");
    }

    #[test]
    fn test_inseam_measurement() {
        let mut bm = new_body_measurement();
        inseam_measurement(&mut bm, 80.0);
        assert_eq!(bm.points[0].name, "inseam");
    }

    #[test]
    fn test_arm_length_measurement() {
        let mut bm = new_body_measurement();
        arm_length_measurement(&mut bm, 60.0);
        assert_eq!(bm.points[0].name, "arm_length");
    }

    #[test]
    fn test_measurements_to_json() {
        let mut bm = new_body_measurement();
        chest_measurement(&mut bm, 90.0);
        let json = measurements_to_json(&bm);
        assert!(json.contains("chest"));
        assert!(json.contains("90.00"));
    }

    #[test]
    fn test_measurement_count_multiple() {
        let mut bm = new_body_measurement();
        chest_measurement(&mut bm, 90.0);
        hip_measurement(&mut bm, 100.0);
        waist_measurement(&mut bm, 70.0);
        assert_eq!(measurement_count(&bm), 3);
    }

    #[test]
    fn test_json_empty() {
        let bm = new_body_measurement();
        assert_eq!(measurements_to_json(&bm), "[]");
    }

    #[test]
    fn test_multiple_same_type() {
        let mut bm = new_body_measurement();
        chest_measurement(&mut bm, 90.0);
        chest_measurement(&mut bm, 95.0);
        assert_eq!(measurement_count(&bm), 2);
    }
}
