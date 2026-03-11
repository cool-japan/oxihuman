//! Particle effect rendering data (billboards, sprites).

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ParticleBlend {
    Additive,
    AlphaBlend,
    Multiply,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct RenderParticle {
    pub position: [f32; 3],
    pub size: f32,
    pub color: [f32; 4],
    pub rotation: f32,
    pub tex_frame: u32,
    pub age: f32,
    pub lifetime: f32,
}

#[allow(dead_code)]
pub struct ParticleSystem {
    pub particles: Vec<RenderParticle>,
    pub blend_mode: ParticleBlend,
    pub texture_id: u32,
    pub sort_by_depth: bool,
    pub max_particles: usize,
    pub camera_position: [f32; 3],
}

#[allow(dead_code)]
pub fn new_particle_system(max: usize, blend: ParticleBlend) -> ParticleSystem {
    ParticleSystem {
        particles: Vec::with_capacity(max),
        blend_mode: blend,
        texture_id: 0,
        sort_by_depth: false,
        max_particles: max,
        camera_position: [0.0, 0.0, 0.0],
    }
}

/// Emit a particle. Returns false if the system is full.
#[allow(dead_code)]
pub fn emit_particle(sys: &mut ParticleSystem, p: RenderParticle) -> bool {
    if sys.particles.len() >= sys.max_particles {
        return false;
    }
    sys.particles.push(p);
    true
}

/// Advance particle ages by `dt` and remove dead particles.
#[allow(dead_code)]
pub fn update_particles(sys: &mut ParticleSystem, dt: f32) {
    for p in sys.particles.iter_mut() {
        p.age += dt;
    }
    sys.particles.retain(|p| p.age < p.lifetime);
}

/// Sort particles back-to-front by distance from the camera.
#[allow(dead_code)]
pub fn sort_particles_by_depth(sys: &mut ParticleSystem) {
    let cam = sys.camera_position;
    sys.particles.sort_by(|a, b| {
        let da = dist_sq(a.position, cam);
        let db = dist_sq(b.position, cam);
        // back-to-front: larger distance first
        db.partial_cmp(&da).unwrap_or(std::cmp::Ordering::Equal)
    });
}

fn dist_sq(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

#[allow(dead_code)]
pub fn particle_count(sys: &ParticleSystem) -> usize {
    sys.particles.len()
}

#[allow(dead_code)]
pub fn alive_particle_count(sys: &ParticleSystem) -> usize {
    sys.particles
        .iter()
        .filter(|p| is_particle_alive(p))
        .count()
}

/// Returns billboard corner positions for each alive particle.
#[allow(dead_code)]
pub fn particles_as_quads(sys: &ParticleSystem) -> Vec<[[f32; 3]; 4]> {
    // Use a default camera-facing right/up for simplicity
    let right = [1.0_f32, 0.0, 0.0];
    let up = [0.0_f32, 1.0, 0.0];
    sys.particles
        .iter()
        .filter(|p| is_particle_alive(p))
        .map(|p| billboard_corners(p.position, p.size, right, up))
        .collect()
}

/// Compute 4 billboard corner positions given position, size, and camera right/up vectors.
#[allow(dead_code)]
pub fn billboard_corners(pos: [f32; 3], size: f32, right: [f32; 3], up: [f32; 3]) -> [[f32; 3]; 4] {
    let half = size * 0.5;
    let r = [right[0] * half, right[1] * half, right[2] * half];
    let u = [up[0] * half, up[1] * half, up[2] * half];
    [
        [
            pos[0] - r[0] - u[0],
            pos[1] - r[1] - u[1],
            pos[2] - r[2] - u[2],
        ], // bottom-left
        [
            pos[0] + r[0] - u[0],
            pos[1] + r[1] - u[1],
            pos[2] + r[2] - u[2],
        ], // bottom-right
        [
            pos[0] + r[0] + u[0],
            pos[1] + r[1] + u[1],
            pos[2] + r[2] + u[2],
        ], // top-right
        [
            pos[0] - r[0] + u[0],
            pos[1] - r[1] + u[1],
            pos[2] - r[2] + u[2],
        ], // top-left
    ]
}

/// Returns (u0, v0, u1, v1) UV rect for a given frame in a sprite atlas.
#[allow(dead_code)]
pub fn particle_uv_frame(frame: u32, atlas_cols: u32, atlas_rows: u32) -> [f32; 4] {
    let cols = atlas_cols.max(1);
    let rows = atlas_rows.max(1);
    let col = frame % cols;
    let row = frame / cols % rows;
    let inv_cols = 1.0 / cols as f32;
    let inv_rows = 1.0 / rows as f32;
    let u0 = col as f32 * inv_cols;
    let v0 = row as f32 * inv_rows;
    let u1 = u0 + inv_cols;
    let v1 = v0 + inv_rows;
    [u0, v0, u1, v1]
}

#[allow(dead_code)]
pub fn clear_particles(sys: &mut ParticleSystem) {
    sys.particles.clear();
}

#[allow(dead_code)]
pub fn is_particle_alive(p: &RenderParticle) -> bool {
    p.age < p.lifetime
}

/// Returns age / lifetime clamped to [0, 1].
#[allow(dead_code)]
pub fn particle_normalized_age(p: &RenderParticle) -> f32 {
    if p.lifetime <= 0.0 {
        return 1.0;
    }
    (p.age / p.lifetime).clamp(0.0, 1.0)
}

#[allow(dead_code)]
pub fn average_particle_age(sys: &ParticleSystem) -> f32 {
    if sys.particles.is_empty() {
        return 0.0;
    }
    let total: f32 = sys.particles.iter().map(|p| p.age).sum();
    total / sys.particles.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_particle(age: f32, lifetime: f32) -> RenderParticle {
        RenderParticle {
            position: [0.0, 0.0, 0.0],
            size: 1.0,
            color: [1.0, 1.0, 1.0, 1.0],
            rotation: 0.0,
            tex_frame: 0,
            age,
            lifetime,
        }
    }

    #[test]
    fn test_new_particle_system() {
        let sys = new_particle_system(100, ParticleBlend::Additive);
        assert_eq!(sys.max_particles, 100);
        assert_eq!(sys.blend_mode, ParticleBlend::Additive);
        assert!(sys.particles.is_empty());
    }

    #[test]
    fn test_emit_particle() {
        let mut sys = new_particle_system(2, ParticleBlend::AlphaBlend);
        let p = make_particle(0.0, 1.0);
        assert!(emit_particle(&mut sys, p.clone()));
        assert!(emit_particle(&mut sys, p.clone()));
        // third should fail - system full
        assert!(!emit_particle(&mut sys, p));
    }

    #[test]
    fn test_update_particles_ages() {
        let mut sys = new_particle_system(10, ParticleBlend::Additive);
        emit_particle(&mut sys, make_particle(0.0, 2.0));
        update_particles(&mut sys, 0.5);
        assert!((sys.particles[0].age - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_update_particles_removes_dead() {
        let mut sys = new_particle_system(10, ParticleBlend::Additive);
        emit_particle(&mut sys, make_particle(0.0, 0.1));
        emit_particle(&mut sys, make_particle(0.0, 5.0));
        update_particles(&mut sys, 0.5);
        assert_eq!(particle_count(&sys), 1);
    }

    #[test]
    fn test_particle_count() {
        let mut sys = new_particle_system(10, ParticleBlend::Additive);
        assert_eq!(particle_count(&sys), 0);
        emit_particle(&mut sys, make_particle(0.0, 1.0));
        assert_eq!(particle_count(&sys), 1);
    }

    #[test]
    fn test_alive_particle_count() {
        let mut sys = new_particle_system(10, ParticleBlend::Additive);
        emit_particle(&mut sys, make_particle(0.5, 1.0)); // alive
        emit_particle(&mut sys, make_particle(2.0, 1.0)); // dead
        assert_eq!(alive_particle_count(&sys), 1);
    }

    #[test]
    fn test_billboard_corners_produces_4_points() {
        let corners = billboard_corners([0.0, 0.0, 0.0], 1.0, [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert_eq!(corners.len(), 4);
        // Verify corners are distinct
        assert_ne!(corners[0], corners[1]);
        assert_ne!(corners[0], corners[2]);
    }

    #[test]
    fn test_particle_uv_frame() {
        // 4x4 atlas, frame 0 -> (0, 0, 0.25, 0.25)
        let uv = particle_uv_frame(0, 4, 4);
        assert!((uv[0] - 0.0).abs() < f32::EPSILON);
        assert!((uv[1] - 0.0).abs() < f32::EPSILON);
        assert!((uv[2] - 0.25).abs() < f32::EPSILON);
        assert!((uv[3] - 0.25).abs() < f32::EPSILON);
    }

    #[test]
    fn test_particle_uv_frame_second() {
        // 4x4 atlas, frame 1 -> (0.25, 0, 0.5, 0.25)
        let uv = particle_uv_frame(1, 4, 4);
        assert!((uv[0] - 0.25).abs() < f32::EPSILON);
        assert!((uv[2] - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_clear_particles() {
        let mut sys = new_particle_system(10, ParticleBlend::Additive);
        emit_particle(&mut sys, make_particle(0.0, 1.0));
        clear_particles(&mut sys);
        assert_eq!(particle_count(&sys), 0);
    }

    #[test]
    fn test_is_particle_alive() {
        let alive = make_particle(0.5, 1.0);
        let dead = make_particle(1.5, 1.0);
        assert!(is_particle_alive(&alive));
        assert!(!is_particle_alive(&dead));
    }

    #[test]
    fn test_particle_normalized_age() {
        let p = make_particle(0.5, 2.0);
        let n = particle_normalized_age(&p);
        assert!((n - 0.25).abs() < f32::EPSILON);
    }

    #[test]
    fn test_particle_normalized_age_clamped() {
        let p = make_particle(5.0, 1.0);
        assert!((particle_normalized_age(&p) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_average_particle_age() {
        let mut sys = new_particle_system(10, ParticleBlend::Additive);
        emit_particle(&mut sys, make_particle(1.0, 5.0));
        emit_particle(&mut sys, make_particle(3.0, 5.0));
        let avg = average_particle_age(&sys);
        assert!((avg - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_average_particle_age_empty() {
        let sys = new_particle_system(10, ParticleBlend::Additive);
        assert!((average_particle_age(&sys)).abs() < f32::EPSILON);
    }
}
