// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A point on a motion trajectory.
pub struct TrajectoryPoint {
    pub time_s: f32,
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub acc: [f32; 3],
}

pub fn new_trajectory_point(t: f32, pos: [f32; 3]) -> TrajectoryPoint {
    TrajectoryPoint {
        time_s: t,
        pos,
        vel: [0.0; 3],
        acc: [0.0; 3],
    }
}

pub fn trajectory_to_csv_line(p: &TrajectoryPoint) -> String {
    format!(
        "{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4}",
        p.time_s,
        p.pos[0],
        p.pos[1],
        p.pos[2],
        p.vel[0],
        p.vel[1],
        p.vel[2],
        p.acc[0],
        p.acc[1],
        p.acc[2],
    )
}

pub fn trajectory_sequence_to_csv(points: &[TrajectoryPoint]) -> String {
    let header = "time_s,px,py,pz,vx,vy,vz,ax,ay,az\n";
    let rows: Vec<String> = points.iter().map(trajectory_to_csv_line).collect();
    format!("{}{}", header, rows.join("\n"))
}

pub fn trajectory_total_distance(points: &[TrajectoryPoint]) -> f32 {
    let n = points.len();
    if n < 2 {
        return 0.0;
    }
    let mut total = 0.0f32;
    for i in 0..n - 1 {
        let a = points[i].pos;
        let b = points[i + 1].pos;
        let dx = b[0] - a[0];
        let dy = b[1] - a[1];
        let dz = b[2] - a[2];
        total += (dx * dx + dy * dy + dz * dz).sqrt();
    }
    total
}

pub fn trajectory_max_speed(points: &[TrajectoryPoint]) -> f32 {
    points
        .iter()
        .map(|p| {
            let v = p.vel;
            (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
        })
        .fold(0.0f32, f32::max)
}

pub fn trajectory_duration(points: &[TrajectoryPoint]) -> f32 {
    if points.len() < 2 {
        return 0.0;
    }
    points.last().map_or(0.0, |p| p.time_s) - points.first().map_or(0.0, |p| p.time_s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_trajectory_point() {
        let p = new_trajectory_point(1.0, [1.0, 2.0, 3.0]);
        assert!((p.time_s - 1.0).abs() < 1e-5);
        assert_eq!(p.vel, [0.0; 3]);
    }

    #[test]
    fn test_trajectory_to_csv_line() {
        let p = new_trajectory_point(0.0, [0.0; 3]);
        let line = trajectory_to_csv_line(&p);
        assert!(line.contains("0.0000"));
    }

    #[test]
    fn test_trajectory_sequence_to_csv_header() {
        let csv = trajectory_sequence_to_csv(&[]);
        assert!(csv.starts_with("time_s"));
    }

    #[test]
    fn test_trajectory_total_distance() {
        let pts = vec![
            new_trajectory_point(0.0, [0.0, 0.0, 0.0]),
            new_trajectory_point(1.0, [3.0, 4.0, 0.0]),
        ];
        assert!((trajectory_total_distance(&pts) - 5.0).abs() < 1e-4);
    }

    #[test]
    fn test_trajectory_max_speed() {
        let mut pts = vec![
            new_trajectory_point(0.0, [0.0; 3]),
            new_trajectory_point(1.0, [0.0; 3]),
        ];
        pts[0].vel = [3.0, 4.0, 0.0];
        pts[1].vel = [1.0, 0.0, 0.0];
        assert!((trajectory_max_speed(&pts) - 5.0).abs() < 1e-4);
    }

    #[test]
    fn test_trajectory_duration() {
        let pts = vec![
            new_trajectory_point(0.5, [0.0; 3]),
            new_trajectory_point(2.5, [0.0; 3]),
        ];
        assert!((trajectory_duration(&pts) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_trajectory_duration_single() {
        let pts = vec![new_trajectory_point(1.0, [0.0; 3])];
        assert!((trajectory_duration(&pts) - 0.0).abs() < 1e-6);
    }
}
