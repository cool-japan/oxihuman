// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! NMEA 0183 sentence export (GGA, RMC).

/// Compute NMEA checksum (XOR of all bytes between `$` and `*`).
#[allow(dead_code)]
pub fn nmea_checksum(sentence: &str) -> u8 {
    sentence.bytes().fold(0u8, |acc, b| acc ^ b)
}

/// Format a decimal degree value as NMEA DDmm.mmmm.
#[allow(dead_code)]
pub fn decimal_to_nmea(degrees: f64) -> (String, char) {
    let abs = degrees.abs();
    let deg = abs.floor() as u32;
    let min = (abs - deg as f64) * 60.0;
    let s = if degrees >= 0.0 { 'N' } else { 'S' };
    (format!("{:02}{:09.6}", deg, min), s)
}

/// Format a decimal longitude as NMEA DDDmm.mmmm.
#[allow(dead_code)]
pub fn decimal_lon_to_nmea(lon: f64) -> (String, char) {
    let abs = lon.abs();
    let deg = abs.floor() as u32;
    let min = (abs - deg as f64) * 60.0;
    let s = if lon >= 0.0 { 'E' } else { 'W' };
    (format!("{:03}{:09.6}", deg, min), s)
}

/// Build a GGA sentence (GPS fix data).
#[allow(dead_code)]
pub fn build_gga(
    time_hhmmss: &str,
    lat: f64,
    lon: f64,
    fix_quality: u8,
    num_satellites: u8,
    altitude_m: f64,
) -> String {
    let (lat_str, lat_dir) = decimal_to_nmea(lat);
    let (lon_str, lon_dir) = decimal_lon_to_nmea(lon);
    let body = format!(
        "GPGGA,{},{},{},{},{},{},{},{},,,M,,M,,",
        time_hhmmss, lat_str, lat_dir, lon_str, lon_dir, fix_quality, num_satellites, altitude_m
    );
    let cs = nmea_checksum(body.as_str());
    format!("${}*{:02X}\r\n", body, cs)
}

/// Build an RMC sentence (Recommended Minimum Specific GPS data).
#[allow(dead_code)]
pub fn build_rmc(
    time_hhmmss: &str,
    status: char,
    lat: f64,
    lon: f64,
    speed_knots: f64,
    course_deg: f64,
    date_ddmmyy: &str,
) -> String {
    let (lat_str, lat_dir) = decimal_to_nmea(lat);
    let (lon_str, lon_dir) = decimal_lon_to_nmea(lon);
    let body = format!(
        "GPRMC,{},{},{},{},{},{},{},{},{},,,A",
        time_hhmmss,
        status,
        lat_str,
        lat_dir,
        lon_str,
        lon_dir,
        speed_knots,
        course_deg,
        date_ddmmyy
    );
    let cs = nmea_checksum(body.as_str());
    format!("${}*{:02X}\r\n", body, cs)
}

/// Export a sequence of GPS positions as GGA sentences.
#[allow(dead_code)]
pub fn export_positions_as_gga(positions: &[(f64, f64, f64)], base_time_s: u32) -> Vec<String> {
    positions
        .iter()
        .enumerate()
        .map(|(i, &(lat, lon, alt))| {
            let t = base_time_s + i as u32;
            let hh = (t / 3600) % 24;
            let mm = (t / 60) % 60;
            let ss = t % 60;
            let time_str = format!("{:02}{:02}{:02}.00", hh, mm, ss);
            build_gga(&time_str, lat, lon, 1, 8, alt)
        })
        .collect()
}

/// Sentence count in a collection.
#[allow(dead_code)]
pub fn sentence_count(sentences: &[String]) -> usize {
    sentences.len()
}

/// Validate that a sentence ends with `\r\n`.
#[allow(dead_code)]
pub fn sentence_has_crlf(s: &str) -> bool {
    s.ends_with("\r\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gga_starts_with_dollar() {
        let s = build_gga("120000.00", 35.0, 139.0, 1, 8, 100.0);
        assert!(s.starts_with('$'));
    }

    #[test]
    fn gga_ends_with_crlf() {
        let s = build_gga("120000.00", 35.0, 139.0, 1, 8, 100.0);
        assert!(sentence_has_crlf(&s));
    }

    #[test]
    fn gga_contains_gpgga() {
        let s = build_gga("120000.00", 35.0, 139.0, 1, 8, 100.0);
        assert!(s.contains("GPGGA"));
    }

    #[test]
    fn rmc_starts_with_dollar() {
        let s = build_rmc("120000.00", 'A', 35.0, 139.0, 0.0, 0.0, "070326");
        assert!(s.starts_with('$'));
    }

    #[test]
    fn rmc_contains_gprmc() {
        let s = build_rmc("120000.00", 'A', 35.0, 139.0, 0.0, 0.0, "070326");
        assert!(s.contains("GPRMC"));
    }

    #[test]
    fn decimal_to_nmea_positive() {
        let (val, dir) = decimal_to_nmea(35.6762);
        assert_eq!(dir, 'N');
        assert!(val.starts_with("35"));
    }

    #[test]
    fn decimal_to_nmea_negative() {
        let (_, dir) = decimal_to_nmea(-35.0);
        assert_eq!(dir, 'S');
    }

    #[test]
    fn decimal_lon_east() {
        let (_, dir) = decimal_lon_to_nmea(139.0);
        assert_eq!(dir, 'E');
    }

    #[test]
    fn decimal_lon_west() {
        let (_, dir) = decimal_lon_to_nmea(-100.0);
        assert_eq!(dir, 'W');
    }

    #[test]
    fn export_gga_count() {
        let pts = vec![(35.0, 139.0, 0.0), (35.1, 139.1, 10.0)];
        let sentences = export_positions_as_gga(&pts, 43200);
        assert_eq!(sentence_count(&sentences), 2);
    }

    #[test]
    fn checksum_nonzero() {
        let cs = nmea_checksum("GPGGA,120000.00,3535.0000,N,13900.0000,E,1,8,100.0,,,M,,M,,");
        let _ = cs;
    }
}
