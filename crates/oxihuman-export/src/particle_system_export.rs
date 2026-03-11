#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Export particle system settings.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ParticleSystemExport {
    pub name: String,
    pub count: u32,
    pub lifetime: f32,
    pub emit_from: u8,
    pub gravity: f32,
    pub velocity: [f32; 3],
    pub size: f32,
    pub enabled: bool,
}

#[allow(dead_code)]
pub fn default_particle_system_export(name: &str) -> ParticleSystemExport {
    ParticleSystemExport {
        name: name.to_string(),
        count: 1000,
        lifetime: 60.0,
        emit_from: 0,
        gravity: -9.81,
        velocity: [0.0, 1.0, 0.0],
        size: 0.05,
        enabled: true,
    }
}

#[allow(dead_code)]
pub fn export_particle_system_to_json(p: &ParticleSystemExport) -> String {
    format!(
        r#"{{"name":"{}","count":{},"lifetime":{},"emit_from":{},"gravity":{},"velocity":[{},{},{}],"size":{},"enabled":{}}}"#,
        p.name, p.count, p.lifetime, p.emit_from, p.gravity,
        p.velocity[0], p.velocity[1], p.velocity[2],
        p.size, p.enabled
    )
}

#[allow(dead_code)]
pub fn particle_system_from_str(s: &str) -> ParticleSystemExport {
    // Minimal stub: extract name if present, else default
    let name = if let Some(start) = s.find("\"name\":\"") {
        let rest = &s[start + 8..];
        if let Some(end) = rest.find('"') {
            &rest[..end]
        } else {
            "unknown"
        }
    } else {
        "unknown"
    };
    default_particle_system_export(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_name() {
        let p = default_particle_system_export("sparks");
        assert_eq!(p.name, "sparks");
    }

    #[test]
    fn default_count_1000() {
        let p = default_particle_system_export("x");
        assert_eq!(p.count, 1000);
    }

    #[test]
    fn default_enabled_true() {
        let p = default_particle_system_export("x");
        assert!(p.enabled);
    }

    #[test]
    fn export_json_has_name() {
        let p = default_particle_system_export("fire");
        let j = export_particle_system_to_json(&p);
        assert!(j.contains("fire"));
    }

    #[test]
    fn export_json_has_count() {
        let p = default_particle_system_export("x");
        let j = export_particle_system_to_json(&p);
        assert!(j.contains("count"));
    }

    #[test]
    fn export_json_has_velocity() {
        let p = default_particle_system_export("x");
        let j = export_particle_system_to_json(&p);
        assert!(j.contains("velocity"));
    }

    #[test]
    fn export_json_has_gravity() {
        let p = default_particle_system_export("x");
        let j = export_particle_system_to_json(&p);
        assert!(j.contains("gravity"));
    }

    #[test]
    fn from_str_roundtrip_name() {
        let p = default_particle_system_export("dust");
        let j = export_particle_system_to_json(&p);
        let p2 = particle_system_from_str(&j);
        assert_eq!(p2.name, "dust");
    }

    #[test]
    fn export_json_has_size() {
        let p = default_particle_system_export("x");
        let j = export_particle_system_to_json(&p);
        assert!(j.contains("size"));
    }
}
