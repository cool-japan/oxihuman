// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

use oxihuman_core::parser::target::Delta;

/// Scatter-add morph: for each delta in target, accumulate dx/dy/dz * weight
/// into the SoA position arrays x[], y[], z[].
///
/// Preconditions: x/y/z have the same length ≥ max(delta.vid).
pub fn apply_target(x: &mut [f32], y: &mut [f32], z: &mut [f32], deltas: &[Delta], weight: f32) {
    if weight == 0.0 {
        return;
    }
    for delta in deltas {
        let i = delta.vid as usize;
        // Safety: callers ensure i < x.len()
        if i < x.len() {
            x[i] += delta.dx * weight;
            y[i] += delta.dy * weight;
            z[i] += delta.dz * weight;
        }
    }
}

/// Reset SoA arrays from base positions stored as AoS (`Vec<[f32;3]>`).
pub fn reset_from_base(x: &mut [f32], y: &mut [f32], z: &mut [f32], base: &[[f32; 3]]) {
    for (i, pos) in base.iter().enumerate() {
        if i < x.len() {
            x[i] = pos[0];
            y[i] = pos[1];
            z[i] = pos[2];
        }
    }
}

/// Combine SoA x/y/z back to AoS `Vec<[f32;3]>`.
pub fn soa_to_aos(x: &[f32], y: &[f32], z: &[f32]) -> Vec<[f32; 3]> {
    x.iter()
        .zip(y)
        .zip(z)
        .map(|((xi, yi), zi)| [*xi, *yi, *zi])
        .collect()
}

use rayon::prelude::*;

/// Parallel scatter-add: divides vertices into rayon chunks, each chunk
/// processes all targets for its vertex range using binary search on sorted deltas.
///
/// Precondition: all `deltas` slices must be sorted by `vid` (guaranteed by `parse_target`).
pub fn apply_targets_parallel(
    x: &mut [f32],
    y: &mut [f32],
    z: &mut [f32],
    targets: &[(&[Delta], f32)],
) {
    if targets.is_empty() {
        return;
    }

    let n = x.len();
    // Chunk size: aim for ~4096 vertices per chunk for cache efficiency
    let chunk_size = (n / rayon::current_num_threads()).clamp(512, 4096);

    // Split into parallel chunks and process each independently
    x.par_chunks_mut(chunk_size)
        .zip(y.par_chunks_mut(chunk_size))
        .zip(z.par_chunks_mut(chunk_size))
        .enumerate()
        .for_each(|(chunk_idx, ((xc, yc), zc))| {
            let base_vid = (chunk_idx * chunk_size) as u32;
            let end_vid = base_vid + xc.len() as u32;

            for &(deltas, weight) in targets {
                if weight == 0.0 {
                    continue;
                }

                // Binary search for start of this chunk's range
                let start = deltas.partition_point(|d| d.vid < base_vid);
                for delta in &deltas[start..] {
                    if delta.vid >= end_vid {
                        break;
                    }
                    let local_i = (delta.vid - base_vid) as usize;
                    xc[local_i] += delta.dx * weight;
                    yc[local_i] += delta.dy * weight;
                    zc[local_i] += delta.dz * weight;
                }
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_core::parser::target::Delta;

    #[test]
    fn apply_single_delta() {
        let mut x = vec![0.0f32; 10];
        let mut y = vec![0.0f32; 10];
        let mut z = vec![0.0f32; 10];
        let deltas = vec![Delta {
            vid: 3,
            dx: 1.0,
            dy: 2.0,
            dz: 3.0,
        }];
        apply_target(&mut x, &mut y, &mut z, &deltas, 0.5);
        assert!((x[3] - 0.5).abs() < 1e-6);
        assert!((y[3] - 1.0).abs() < 1e-6);
        assert!((z[3] - 1.5).abs() < 1e-6);
    }

    #[test]
    fn zero_weight_is_noop() {
        let mut x = vec![5.0f32; 5];
        let mut y = vec![5.0f32; 5];
        let mut z = vec![5.0f32; 5];
        let deltas = vec![Delta {
            vid: 0,
            dx: 100.0,
            dy: 100.0,
            dz: 100.0,
        }];
        apply_target(&mut x, &mut y, &mut z, &deltas, 0.0);
        assert!((x[0] - 5.0).abs() < 1e-6);
    }

    #[test]
    fn soa_roundtrip() {
        let base = vec![[1.0f32, 2.0, 3.0], [4.0, 5.0, 6.0]];
        let mut x = vec![0.0f32; 2];
        let mut y = vec![0.0f32; 2];
        let mut z = vec![0.0f32; 2];
        reset_from_base(&mut x, &mut y, &mut z, &base);
        let out = soa_to_aos(&x, &y, &z);
        assert_eq!(out, base);
    }

    #[test]
    fn parallel_apply_matches_sequential() {
        let n = 1000usize;
        let deltas: Vec<Delta> = (0..200u32)
            .map(|i| Delta {
                vid: i * 4,
                dx: 0.1,
                dy: 0.2,
                dz: 0.3,
            })
            .collect();

        let mut x_seq = vec![0.0f32; n];
        let mut y_seq = vec![0.0f32; n];
        let mut z_seq = vec![0.0f32; n];
        apply_target(&mut x_seq, &mut y_seq, &mut z_seq, &deltas, 0.5);

        let mut x_par = vec![0.0f32; n];
        let mut y_par = vec![0.0f32; n];
        let mut z_par = vec![0.0f32; n];
        let targets = vec![(deltas.as_slice(), 0.5f32)];
        apply_targets_parallel(&mut x_par, &mut y_par, &mut z_par, &targets);

        for i in 0..n {
            assert!((x_seq[i] - x_par[i]).abs() < 1e-6, "x mismatch at {}", i);
            assert!((y_seq[i] - y_par[i]).abs() < 1e-6, "y mismatch at {}", i);
            assert!((z_seq[i] - z_par[i]).abs() < 1e-6, "z mismatch at {}", i);
        }
    }

    #[test]
    fn parallel_apply_multiple_targets() {
        let n = 500usize;
        let d1: Vec<Delta> = (0..50u32)
            .map(|i| Delta {
                vid: i * 2,
                dx: 1.0,
                dy: 0.0,
                dz: 0.0,
            })
            .collect();
        let d2: Vec<Delta> = (0..50u32)
            .map(|i| Delta {
                vid: i * 2,
                dx: 0.0,
                dy: 1.0,
                dz: 0.0,
            })
            .collect();

        let mut x = vec![0.0f32; n];
        let mut y = vec![0.0f32; n];
        let mut z = vec![0.0f32; n];
        let targets = vec![(d1.as_slice(), 1.0f32), (d2.as_slice(), 0.5f32)];
        apply_targets_parallel(&mut x, &mut y, &mut z, &targets);

        // vid=0: x += 1.0*1.0=1.0, y += 1.0*0.5=0.5
        assert!((x[0] - 1.0).abs() < 1e-6);
        assert!((y[0] - 0.5).abs() < 1e-6);
    }
}
