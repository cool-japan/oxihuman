// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Animation frame recording/playback and particle system methods for `WasmEngine`.

use crate::engine_core::{Particle, ParticleSystem, WasmEngine};

impl WasmEngine {
    // -- Animation streaming --

    /// Snapshot current params as a keyframe and append it to the animation clip.
    pub fn record_anim_frame(&mut self) {
        let mut snapshot = std::collections::HashMap::new();
        snapshot.insert("height".to_string(), self.params.height);
        snapshot.insert("weight".to_string(), self.params.weight);
        snapshot.insert("muscle".to_string(), self.params.muscle);
        snapshot.insert("age".to_string(), self.params.age);
        for (k, v) in &self.params.extra {
            snapshot.insert(k.clone(), *v);
        }
        self.anim_frames.push(snapshot);
    }

    /// Clear all recorded animation keyframes.
    pub fn clear_anim_frames(&mut self) {
        self.anim_frames.clear();
        self.anim_current_frame = 0;
        self.anim_accum = 0.0;
    }

    /// Return the number of recorded animation keyframes.
    pub fn anim_frame_count(&self) -> u32 {
        self.anim_frames.len() as u32
    }

    /// Seek to the given frame index: restore the params snapshot from that frame.
    /// If the frame index is out of range, this is a no-op.
    pub fn seek_anim_frame(&mut self, frame: u32) {
        let idx = frame as usize;
        if idx >= self.anim_frames.len() {
            return;
        }
        self.anim_current_frame = idx;
        let snap = self.anim_frames[idx].clone();
        let mut p = self.params.clone();
        if let Some(&v) = snap.get("height") {
            p.height = v;
        }
        if let Some(&v) = snap.get("weight") {
            p.weight = v;
        }
        if let Some(&v) = snap.get("muscle") {
            p.muscle = v;
        }
        if let Some(&v) = snap.get("age") {
            p.age = v;
        }
        for (k, v) in &snap {
            match k.as_str() {
                "height" | "weight" | "muscle" | "age" => {}
                _ => {
                    p.extra.insert(k.clone(), *v);
                }
            }
        }
        self.engine.set_params(p.clone());
        self.params = p;
        self.last_mesh = None;
    }

    /// Advance animation by `dt_seconds` at the current FPS; wrap around.
    /// Returns the new frame index. No-op (returns 0) when there are no frames.
    pub fn play_anim_step(&mut self, dt_seconds: f32) -> u32 {
        let n = self.anim_frames.len();
        if n == 0 {
            return 0;
        }
        self.anim_accum += dt_seconds * self.anim_fps;
        let steps = self.anim_accum.floor() as usize;
        self.anim_accum -= steps as f32;
        self.anim_current_frame = (self.anim_current_frame + steps) % n;
        let frame = self.anim_current_frame as u32;
        self.seek_anim_frame(frame);
        frame
    }

    /// Set the animation playback speed in frames per second.
    pub fn set_anim_fps(&mut self, fps: f32) {
        self.anim_fps = fps.max(0.0);
    }

    /// Return the current animation FPS.
    pub fn get_anim_fps(&self) -> f32 {
        self.anim_fps
    }

    /// Serialize all recorded animation frames as a JSON array of param objects.
    ///
    /// Example: `[{"height":0.5,"weight":0.5,...}, ...]`
    pub fn export_anim_json(&self) -> String {
        let frames: Vec<String> = self
            .anim_frames
            .iter()
            .map(|snap| {
                let pairs: Vec<String> = snap
                    .iter()
                    .map(|(k, v)| {
                        let k_esc = k.replace('\\', "\\\\").replace('"', "\\\"");
                        format!("\"{}\":{}", k_esc, v)
                    })
                    .collect();
                format!("{{{}}}", pairs.join(","))
            })
            .collect();
        format!("[{}]", frames.join(","))
    }

    // -- Particle system --

    /// Create a default point emitter and store it in engine state.
    /// Returns `true` on success.
    pub fn create_particle_system(&mut self, emit_rate: f32, lifetime: f32) -> bool {
        self.particle_sys = Some(ParticleSystem {
            emit_rate,
            lifetime,
            particles: Vec::new(),
            time_accum: 0.0,
        });
        true
    }

    /// Advance the particle simulation by `dt` seconds.
    /// Returns JSON: `{"active": N, "positions": [[x,y,z], ...]}`.
    pub fn step_particles(&mut self, dt: f32) -> String {
        let ps = match &mut self.particle_sys {
            Some(ps) => ps,
            None => return "{\"active\":0,\"positions\":[]}".to_string(),
        };

        // Age and remove dead particles.
        ps.particles.retain_mut(|p| {
            p.age += dt;
            p.position[0] += p.velocity[0] * dt;
            p.position[1] += p.velocity[1] * dt;
            p.position[2] += p.velocity[2] * dt;
            p.age < p.lifetime
        });

        // Emit new particles.
        ps.time_accum += dt;
        let interval = if ps.emit_rate > 0.0 {
            1.0 / ps.emit_rate
        } else {
            f32::MAX
        };
        while ps.time_accum >= interval {
            ps.time_accum -= interval;
            // Emit at origin with a small upward velocity.
            ps.particles.push(Particle {
                position: [0.0, 0.0, 0.0],
                velocity: [0.0, 1.0, 0.0],
                age: 0.0,
                lifetime: ps.lifetime,
            });
        }

        let positions: Vec<[f32; 3]> = ps.particles.iter().map(|p| p.position).collect();
        let active = positions.len();
        serde_json::json!({
            "active": active,
            "positions": positions
        })
        .to_string()
    }
}
