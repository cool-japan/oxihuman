// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Center of mass (CoM) trajectory planner stub.

/// A sampled CoM trajectory waypoint.
#[derive(Debug, Clone, PartialEq)]
pub struct ComWaypoint {
    pub time: f32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// CoM trajectory (ordered list of waypoints).
#[derive(Debug, Clone, Default)]
pub struct ComTrajectory {
    pub waypoints: Vec<ComWaypoint>,
}

impl ComTrajectory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_waypoint(&mut self, wp: ComWaypoint) {
        self.waypoints.push(wp);
        self.waypoints.sort_by(|a, b| {
            a.time
                .partial_cmp(&b.time)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    pub fn len(&self) -> usize {
        self.waypoints.len()
    }

    pub fn is_empty(&self) -> bool {
        self.waypoints.is_empty()
    }
}

/// Linearly interpolate the CoM position at time `t`.
pub fn sample_com_trajectory(traj: &ComTrajectory, t: f32) -> Option<[f32; 3]> {
    if traj.waypoints.is_empty() {
        return None;
    }
    /* clamp to trajectory ends */
    if t <= traj.waypoints[0].time {
        let w = &traj.waypoints[0];
        return Some([w.x, w.y, w.z]);
    }
    let last = &traj.waypoints[traj.waypoints.len() - 1];
    if t >= last.time {
        return Some([last.x, last.y, last.z]);
    }
    /* find surrounding segment */
    for i in 0..traj.waypoints.len().saturating_sub(1) {
        let a = &traj.waypoints[i];
        let b = &traj.waypoints[i + 1];
        if (a.time..=b.time).contains(&t) {
            let span = (b.time - a.time).max(1e-8);
            let alpha = (t - a.time) / span;
            return Some([
                a.x + alpha * (b.x - a.x),
                a.y + alpha * (b.y - a.y),
                a.z + alpha * (b.z - a.z),
            ]);
        }
    }
    None
}

/// Compute the average CoM height across the trajectory.
pub fn average_com_height(traj: &ComTrajectory) -> f32 {
    if traj.is_empty() {
        return 0.0;
    }
    traj.waypoints.iter().map(|w| w.z).sum::<f32>() / traj.len() as f32
}

/// Generate a straight-line CoM trajectory between two points.
pub fn linear_com_trajectory(
    start: [f32; 3],
    end: [f32; 3],
    n: usize,
    duration: f32,
) -> ComTrajectory {
    let mut traj = ComTrajectory::new();
    if n == 0 {
        return traj;
    }
    for i in 0..n {
        let alpha = if n == 1 {
            0.0
        } else {
            i as f32 / (n - 1) as f32
        };
        let t = alpha * duration;
        traj.add_waypoint(ComWaypoint {
            time: t,
            x: start[0] + alpha * (end[0] - start[0]),
            y: start[1] + alpha * (end[1] - start[1]),
            z: start[2] + alpha * (end[2] - start[2]),
        });
    }
    traj
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_trajectory() {
        /* empty trajectory returns None */
        let t = ComTrajectory::new();
        assert!(sample_com_trajectory(&t, 0.0).is_none());
    }

    #[test]
    fn test_single_waypoint() {
        /* single waypoint always returns that point */
        let mut t = ComTrajectory::new();
        t.add_waypoint(ComWaypoint {
            time: 0.0,
            x: 1.0,
            y: 2.0,
            z: 3.0,
        });
        let p = sample_com_trajectory(&t, 5.0).unwrap();
        assert_eq!(p, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_linear_trajectory_len() {
        /* linear trajectory has correct number of waypoints */
        let traj = linear_com_trajectory([0.0; 3], [1.0; 3], 5, 1.0);
        assert_eq!(traj.len(), 5);
    }

    #[test]
    fn test_linear_trajectory_start() {
        /* first waypoint is at start */
        let traj = linear_com_trajectory([1.0, 2.0, 3.0], [4.0, 5.0, 6.0], 5, 1.0);
        let p = sample_com_trajectory(&traj, 0.0).unwrap();
        assert!((p[0] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_linear_trajectory_end() {
        /* last waypoint is at end */
        let traj = linear_com_trajectory([0.0; 3], [1.0, 0.0, 0.0], 5, 1.0);
        let p = sample_com_trajectory(&traj, 1.0).unwrap();
        assert!((p[0] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_sample_midpoint() {
        /* midpoint interpolation */
        let traj = linear_com_trajectory([0.0, 0.0, 0.0], [2.0, 0.0, 0.0], 3, 2.0);
        let p = sample_com_trajectory(&traj, 1.0).unwrap();
        assert!((p[0] - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_average_height() {
        /* average height of flat trajectory */
        let traj = linear_com_trajectory([0.0, 0.0, 1.0], [0.0, 0.0, 1.0], 5, 1.0);
        assert!((average_com_height(&traj) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_trajectory_is_sorted() {
        /* waypoints are sorted by time */
        let mut traj = ComTrajectory::new();
        traj.add_waypoint(ComWaypoint {
            time: 1.0,
            x: 0.0,
            y: 0.0,
            z: 0.0,
        });
        traj.add_waypoint(ComWaypoint {
            time: 0.0,
            x: 0.0,
            y: 0.0,
            z: 0.0,
        });
        assert!(traj.waypoints[0].time <= traj.waypoints[1].time);
    }

    #[test]
    fn test_empty_average_height() {
        /* empty trajectory has zero average height */
        let t = ComTrajectory::new();
        assert_eq!(average_com_height(&t), 0.0);
    }
}
