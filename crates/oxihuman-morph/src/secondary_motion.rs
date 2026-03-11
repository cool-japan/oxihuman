//! Secondary motion — lag/follow-through for hair, clothing, and soft attachments.

#[allow(dead_code)]
pub struct SecondaryBone {
    pub id: u32,
    pub name: String,
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub target_position: [f32; 3],
    pub stiffness: f32,
    pub damping: f32,
    pub mass: f32,
}

#[allow(dead_code)]
pub struct SecondaryChain {
    pub name: String,
    pub bones: Vec<SecondaryBone>,
    pub gravity: [f32; 3],
    pub wind: [f32; 3],
}

#[allow(dead_code)]
pub struct SecondaryMotionConfig {
    pub gravity: [f32; 3],
    pub default_stiffness: f32,
    pub default_damping: f32,
    pub default_mass: f32,
}

#[allow(dead_code)]
pub fn default_secondary_config() -> SecondaryMotionConfig {
    SecondaryMotionConfig {
        gravity: [0.0, -9.81, 0.0],
        default_stiffness: 50.0,
        default_damping: 5.0,
        default_mass: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_secondary_bone(
    id: u32,
    name: &str,
    pos: [f32; 3],
    stiffness: f32,
    damping: f32,
) -> SecondaryBone {
    SecondaryBone {
        id,
        name: name.to_string(),
        position: pos,
        velocity: [0.0, 0.0, 0.0],
        target_position: pos,
        stiffness,
        damping,
        mass: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_secondary_chain(name: &str, gravity: [f32; 3]) -> SecondaryChain {
    SecondaryChain {
        name: name.to_string(),
        bones: Vec::new(),
        gravity,
        wind: [0.0, 0.0, 0.0],
    }
}

#[allow(dead_code)]
pub fn add_secondary_bone(chain: &mut SecondaryChain, bone: SecondaryBone) {
    chain.bones.push(bone);
}

#[allow(dead_code)]
pub fn update_secondary_bone(bone: &mut SecondaryBone, dt: f32, external_force: [f32; 3]) {
    // Spring-damper: F = k*(target-pos) - d*vel + ext
    let spring = [
        bone.stiffness * (bone.target_position[0] - bone.position[0]),
        bone.stiffness * (bone.target_position[1] - bone.position[1]),
        bone.stiffness * (bone.target_position[2] - bone.position[2]),
    ];
    let damp = [
        bone.damping * bone.velocity[0],
        bone.damping * bone.velocity[1],
        bone.damping * bone.velocity[2],
    ];
    let force = [
        spring[0] - damp[0] + external_force[0],
        spring[1] - damp[1] + external_force[1],
        spring[2] - damp[2] + external_force[2],
    ];
    let inv_mass = if bone.mass > 0.0 {
        1.0 / bone.mass
    } else {
        1.0
    };
    bone.velocity[0] += force[0] * inv_mass * dt;
    bone.velocity[1] += force[1] * inv_mass * dt;
    bone.velocity[2] += force[2] * inv_mass * dt;
    bone.position[0] += bone.velocity[0] * dt;
    bone.position[1] += bone.velocity[1] * dt;
    bone.position[2] += bone.velocity[2] * dt;
}

#[allow(dead_code)]
pub fn update_secondary_chain(chain: &mut SecondaryChain, dt: f32, target_positions: &[[f32; 3]]) {
    let gravity = chain.gravity;
    let wind = chain.wind;
    let external = [
        gravity[0] + wind[0],
        gravity[1] + wind[1],
        gravity[2] + wind[2],
    ];
    for (i, bone) in chain.bones.iter_mut().enumerate() {
        if i < target_positions.len() {
            bone.target_position = target_positions[i];
        }
        update_secondary_bone(bone, dt, external);
    }
}

#[allow(dead_code)]
pub fn set_chain_wind(chain: &mut SecondaryChain, wind: [f32; 3]) {
    chain.wind = wind;
}

#[allow(dead_code)]
pub fn secondary_bone_count(chain: &SecondaryChain) -> usize {
    chain.bones.len()
}

#[allow(dead_code)]
pub fn chain_kinetic_energy(chain: &SecondaryChain) -> f32 {
    chain.bones.iter().fold(0.0_f32, |acc, bone| {
        let v2 = bone.velocity[0] * bone.velocity[0]
            + bone.velocity[1] * bone.velocity[1]
            + bone.velocity[2] * bone.velocity[2];
        acc + 0.5 * bone.mass * v2
    })
}

#[allow(dead_code)]
pub fn reset_secondary_chain(chain: &mut SecondaryChain) {
    for bone in chain.bones.iter_mut() {
        bone.velocity = [0.0, 0.0, 0.0];
    }
}

#[allow(dead_code)]
pub fn secondary_chain_positions(chain: &SecondaryChain) -> Vec<[f32; 3]> {
    chain.bones.iter().map(|b| b.position).collect()
}

#[allow(dead_code)]
pub fn secondary_bone_lag(bone: &SecondaryBone) -> f32 {
    let dx = bone.position[0] - bone.target_position[0];
    let dy = bone.position[1] - bone.target_position[1];
    let dz = bone.position[2] - bone.target_position[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[allow(dead_code)]
pub fn blend_secondary_to_target(chain: &mut SecondaryChain, alpha: f32) {
    let a = alpha.clamp(0.0, 1.0);
    for bone in chain.bones.iter_mut() {
        bone.position[0] = bone.position[0] + a * (bone.target_position[0] - bone.position[0]);
        bone.position[1] = bone.position[1] + a * (bone.target_position[1] - bone.position[1]);
        bone.position[2] = bone.position[2] + a * (bone.target_position[2] - bone.position[2]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_secondary_bone_fields() {
        let bone = new_secondary_bone(1, "hair_tip", [1.0, 2.0, 3.0], 40.0, 4.0);
        assert_eq!(bone.id, 1);
        assert_eq!(bone.name, "hair_tip");
        assert_eq!(bone.position, [1.0, 2.0, 3.0]);
        assert_eq!(bone.velocity, [0.0, 0.0, 0.0]);
        assert_eq!(bone.stiffness, 40.0);
        assert_eq!(bone.damping, 4.0);
    }

    #[test]
    fn test_new_secondary_chain_empty() {
        let chain = new_secondary_chain("hair_chain", [0.0, -9.81, 0.0]);
        assert_eq!(chain.name, "hair_chain");
        assert!(chain.bones.is_empty());
    }

    #[test]
    fn test_add_secondary_bone() {
        let mut chain = new_secondary_chain("chain", [0.0, -9.81, 0.0]);
        let bone = new_secondary_bone(0, "b0", [0.0, 0.0, 0.0], 50.0, 5.0);
        add_secondary_bone(&mut chain, bone);
        assert_eq!(chain.bones.len(), 1);
    }

    #[test]
    fn test_secondary_bone_count() {
        let mut chain = new_secondary_chain("c", [0.0, 0.0, 0.0]);
        add_secondary_bone(&mut chain, new_secondary_bone(0, "b0", [0.0; 3], 1.0, 1.0));
        add_secondary_bone(&mut chain, new_secondary_bone(1, "b1", [1.0; 3], 1.0, 1.0));
        assert_eq!(secondary_bone_count(&chain), 2);
    }

    #[test]
    fn test_update_secondary_bone_spring_force() {
        let mut bone = new_secondary_bone(0, "b", [0.0, 0.0, 0.0], 100.0, 0.0);
        bone.target_position = [1.0, 0.0, 0.0];
        // F = k*(target-pos) - d*vel + ext = 100*(1-0) - 0 + 0 = 100
        // a = F/m = 100
        // vel += a*dt => vel = 100 * 0.01 = 1.0
        // pos += vel*dt => pos = 1.0 * 0.01 = 0.01
        update_secondary_bone(&mut bone, 0.01, [0.0, 0.0, 0.0]);
        assert!((bone.velocity[0] - 1.0).abs() < 1e-4);
        assert!((bone.position[0] - 0.01).abs() < 1e-4);
    }

    #[test]
    fn test_update_secondary_bone_external_force() {
        let mut bone = new_secondary_bone(0, "b", [0.0; 3], 0.0, 0.0);
        // external force of 10 in Y, no spring or damping
        update_secondary_bone(&mut bone, 0.1, [0.0, 10.0, 0.0]);
        assert!((bone.velocity[1] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_chain_kinetic_energy_zero_velocity() {
        let mut chain = new_secondary_chain("c", [0.0; 3]);
        add_secondary_bone(&mut chain, new_secondary_bone(0, "b0", [0.0; 3], 1.0, 1.0));
        assert_eq!(chain_kinetic_energy(&chain), 0.0);
    }

    #[test]
    fn test_chain_kinetic_energy_nonzero() {
        let mut chain = new_secondary_chain("c", [0.0; 3]);
        let mut bone = new_secondary_bone(0, "b0", [0.0; 3], 1.0, 1.0);
        bone.velocity = [2.0, 0.0, 0.0];
        bone.mass = 1.0;
        add_secondary_bone(&mut chain, bone);
        // KE = 0.5 * 1.0 * 4.0 = 2.0
        assert!((chain_kinetic_energy(&chain) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_reset_secondary_chain_zeros_velocity() {
        let mut chain = new_secondary_chain("c", [0.0; 3]);
        let mut bone = new_secondary_bone(0, "b0", [0.0; 3], 1.0, 1.0);
        bone.velocity = [5.0, 3.0, 1.0];
        add_secondary_bone(&mut chain, bone);
        reset_secondary_chain(&mut chain);
        assert_eq!(chain.bones[0].velocity, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_secondary_bone_lag() {
        let mut bone = new_secondary_bone(0, "b", [0.0; 3], 1.0, 1.0);
        bone.target_position = [3.0, 4.0, 0.0];
        let lag = secondary_bone_lag(&bone);
        assert!((lag - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_secondary_bone_lag_zero() {
        let bone = new_secondary_bone(0, "b", [1.0, 2.0, 3.0], 1.0, 1.0);
        assert_eq!(secondary_bone_lag(&bone), 0.0);
    }

    #[test]
    fn test_blend_secondary_to_target_full() {
        let mut chain = new_secondary_chain("c", [0.0; 3]);
        let mut bone = new_secondary_bone(0, "b0", [0.0; 3], 1.0, 1.0);
        bone.target_position = [10.0, 0.0, 0.0];
        add_secondary_bone(&mut chain, bone);
        blend_secondary_to_target(&mut chain, 1.0);
        assert!((chain.bones[0].position[0] - 10.0).abs() < 1e-5);
    }

    #[test]
    fn test_blend_secondary_to_target_half() {
        let mut chain = new_secondary_chain("c", [0.0; 3]);
        let mut bone = new_secondary_bone(0, "b0", [0.0; 3], 1.0, 1.0);
        bone.target_position = [10.0, 0.0, 0.0];
        add_secondary_bone(&mut chain, bone);
        blend_secondary_to_target(&mut chain, 0.5);
        assert!((chain.bones[0].position[0] - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_secondary_chain_positions() {
        let mut chain = new_secondary_chain("c", [0.0; 3]);
        add_secondary_bone(
            &mut chain,
            new_secondary_bone(0, "b0", [1.0, 2.0, 3.0], 1.0, 1.0),
        );
        add_secondary_bone(
            &mut chain,
            new_secondary_bone(1, "b1", [4.0, 5.0, 6.0], 1.0, 1.0),
        );
        let positions = secondary_chain_positions(&chain);
        assert_eq!(positions.len(), 2);
        assert_eq!(positions[0], [1.0, 2.0, 3.0]);
        assert_eq!(positions[1], [4.0, 5.0, 6.0]);
    }

    #[test]
    fn test_set_chain_wind() {
        let mut chain = new_secondary_chain("c", [0.0; 3]);
        set_chain_wind(&mut chain, [1.0, 0.0, 2.0]);
        assert_eq!(chain.wind, [1.0, 0.0, 2.0]);
    }

    #[test]
    fn test_update_secondary_chain_targets() {
        let mut chain = new_secondary_chain("c", [0.0; 3]);
        add_secondary_bone(&mut chain, new_secondary_bone(0, "b0", [0.0; 3], 10.0, 0.0));
        let targets = [[1.0, 0.0, 0.0]];
        update_secondary_chain(&mut chain, 0.016, &targets);
        assert_eq!(chain.bones[0].target_position, [1.0, 0.0, 0.0]);
    }

    #[test]
    fn test_default_secondary_config() {
        let cfg = default_secondary_config();
        assert!(cfg.default_stiffness > 0.0);
        assert!(cfg.default_damping > 0.0);
        assert!(cfg.default_mass > 0.0);
        assert!(cfg.gravity[1] < 0.0);
    }
}
