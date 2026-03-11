/// Capsule-based character controller with step handling and movement.
#[allow(dead_code)]
pub struct CharacterCapsule {
    pub radius: f32,
    pub height: f32,
}

#[allow(dead_code)]
pub struct CharacterState {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub on_ground: bool,
    pub slope_angle_deg: f32,
    pub last_normal: [f32; 3],
}

#[allow(dead_code)]
pub struct CharacterController {
    pub capsule: CharacterCapsule,
    pub state: CharacterState,
    pub gravity: [f32; 3],
    pub max_slope_deg: f32,
    pub step_height: f32,
    pub jump_speed: f32,
    pub move_speed: f32,
    pub enabled: bool,
}

#[allow(dead_code)]
pub fn new_character_controller(radius: f32, height: f32) -> CharacterController {
    CharacterController {
        capsule: CharacterCapsule { radius, height },
        state: CharacterState {
            position: [0.0; 3],
            velocity: [0.0; 3],
            on_ground: false,
            slope_angle_deg: 0.0,
            last_normal: [0.0, 1.0, 0.0],
        },
        gravity: [0.0, -9.81, 0.0],
        max_slope_deg: 45.0,
        step_height: 0.3,
        jump_speed: 5.0,
        move_speed: 5.0,
        enabled: true,
    }
}

fn vec3_len(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn vec3_normalize(v: [f32; 3]) -> [f32; 3] {
    let len = vec3_len(v);
    if len < 1e-9 {
        return [0.0; 3];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

/// Move the character in a given direction (normalized), scaled by move_speed and dt.
#[allow(dead_code)]
pub fn move_character(ctrl: &mut CharacterController, direction: [f32; 3], dt: f32) {
    if !ctrl.enabled {
        return;
    }
    let dir = vec3_normalize(direction);
    let speed = ctrl.move_speed;
    ctrl.state.position[0] += dir[0] * speed * dt;
    ctrl.state.position[1] += dir[1] * speed * dt;
    ctrl.state.position[2] += dir[2] * speed * dt;
    ctrl.state.velocity[0] = dir[0] * speed;
    ctrl.state.velocity[1] = dir[1] * speed;
    ctrl.state.velocity[2] = dir[2] * speed;
}

/// Make the character jump by setting upward velocity.
#[allow(dead_code)]
pub fn jump_character(ctrl: &mut CharacterController) {
    if !ctrl.enabled || !ctrl.state.on_ground {
        return;
    }
    ctrl.state.velocity[1] = ctrl.jump_speed;
    ctrl.state.on_ground = false;
}

/// Apply gravity each frame.
#[allow(dead_code)]
pub fn apply_gravity_cc(ctrl: &mut CharacterController, dt: f32) {
    if !ctrl.enabled || ctrl.state.on_ground {
        return;
    }
    ctrl.state.velocity[0] += ctrl.gravity[0] * dt;
    ctrl.state.velocity[1] += ctrl.gravity[1] * dt;
    ctrl.state.velocity[2] += ctrl.gravity[2] * dt;
    ctrl.state.position[0] += ctrl.state.velocity[0] * dt;
    ctrl.state.position[1] += ctrl.state.velocity[1] * dt;
    ctrl.state.position[2] += ctrl.state.velocity[2] * dt;
}

/// Returns true if the character's foot is at or below floor_y.
#[allow(dead_code)]
pub fn ground_check(ctrl: &CharacterController, floor_y: f32) -> bool {
    character_foot_position(ctrl)[1] <= floor_y + 0.01
}

/// Land the character on the given floor.
#[allow(dead_code)]
pub fn land_character(ctrl: &mut CharacterController, floor_y: f32) {
    let foot_offset = ctrl.capsule.height / 2.0;
    ctrl.state.position[1] = floor_y + foot_offset;
    ctrl.state.velocity[1] = 0.0;
    ctrl.state.on_ground = true;
}

/// Returns (min, max) AABB of the capsule.
#[allow(dead_code)]
pub fn capsule_aabb(ctrl: &CharacterController) -> ([f32; 3], [f32; 3]) {
    let pos = ctrl.state.position;
    let r = ctrl.capsule.radius;
    let half_h = ctrl.capsule.height / 2.0;
    let min = [pos[0] - r, pos[1] - half_h, pos[2] - r];
    let max = [pos[0] + r, pos[1] + half_h, pos[2] + r];
    (min, max)
}

/// Returns the position of the character's feet.
#[allow(dead_code)]
pub fn character_foot_position(ctrl: &CharacterController) -> [f32; 3] {
    let pos = ctrl.state.position;
    [pos[0], pos[1] - ctrl.capsule.height / 2.0, pos[2]]
}

/// Returns the position of the character's head.
#[allow(dead_code)]
pub fn character_head_position(ctrl: &CharacterController) -> [f32; 3] {
    let pos = ctrl.state.position;
    [pos[0], pos[1] + ctrl.capsule.height / 2.0, pos[2]]
}

/// Returns the scalar speed of the character.
#[allow(dead_code)]
pub fn character_speed(ctrl: &CharacterController) -> f32 {
    vec3_len(ctrl.state.velocity)
}

/// Returns true if the obstacle height is within the step threshold.
#[allow(dead_code)]
pub fn can_step_up(ctrl: &CharacterController, obstacle_height: f32) -> bool {
    obstacle_height <= ctrl.step_height
}

/// Apply an impulse to the character's velocity.
#[allow(dead_code)]
pub fn push_character(ctrl: &mut CharacterController, impulse: [f32; 3]) {
    ctrl.state.velocity[0] += impulse[0];
    ctrl.state.velocity[1] += impulse[1];
    ctrl.state.velocity[2] += impulse[2];
}

#[allow(dead_code)]
pub fn enable_controller(ctrl: &mut CharacterController) {
    ctrl.enabled = true;
}

#[allow(dead_code)]
pub fn disable_controller(ctrl: &mut CharacterController) {
    ctrl.enabled = false;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_controller() {
        let ctrl = new_character_controller(0.4, 1.8);
        assert!((ctrl.capsule.radius - 0.4).abs() < 1e-6);
        assert!((ctrl.capsule.height - 1.8).abs() < 1e-6);
        assert!(ctrl.enabled);
        assert!(!ctrl.state.on_ground);
    }

    #[test]
    fn test_move_character() {
        let mut ctrl = new_character_controller(0.4, 1.8);
        let start = ctrl.state.position;
        move_character(&mut ctrl, [1.0, 0.0, 0.0], 1.0);
        assert!(ctrl.state.position[0] > start[0]);
    }

    #[test]
    fn test_move_character_disabled() {
        let mut ctrl = new_character_controller(0.4, 1.8);
        disable_controller(&mut ctrl);
        let start = ctrl.state.position;
        move_character(&mut ctrl, [1.0, 0.0, 0.0], 1.0);
        assert_eq!(ctrl.state.position[0], start[0]);
    }

    #[test]
    fn test_jump_character() {
        let mut ctrl = new_character_controller(0.4, 1.8);
        ctrl.state.on_ground = true;
        jump_character(&mut ctrl);
        assert!(!ctrl.state.on_ground);
        assert!(ctrl.state.velocity[1] > 0.0);
    }

    #[test]
    fn test_jump_not_on_ground() {
        let mut ctrl = new_character_controller(0.4, 1.8);
        ctrl.state.on_ground = false;
        ctrl.state.velocity[1] = 0.0;
        jump_character(&mut ctrl);
        assert_eq!(ctrl.state.velocity[1], 0.0); // no jump
    }

    #[test]
    fn test_apply_gravity_cc() {
        let mut ctrl = new_character_controller(0.4, 1.8);
        ctrl.state.on_ground = false;
        let start_y = ctrl.state.position[1];
        apply_gravity_cc(&mut ctrl, 0.1);
        // position should move downward
        assert!(ctrl.state.position[1] < start_y);
    }

    #[test]
    fn test_apply_gravity_on_ground() {
        let mut ctrl = new_character_controller(0.4, 1.8);
        ctrl.state.on_ground = true;
        let start_y = ctrl.state.position[1];
        apply_gravity_cc(&mut ctrl, 0.1);
        assert_eq!(ctrl.state.position[1], start_y);
    }

    #[test]
    fn test_ground_check() {
        let mut ctrl = new_character_controller(0.4, 1.8);
        ctrl.state.position[1] = 0.9; // foot at 0.9 - 0.9 = 0.0
        assert!(ground_check(&ctrl, 0.0));
        ctrl.state.position[1] = 5.0;
        assert!(!ground_check(&ctrl, 0.0));
    }

    #[test]
    fn test_land_character() {
        let mut ctrl = new_character_controller(0.4, 1.8);
        ctrl.state.velocity[1] = -5.0;
        land_character(&mut ctrl, 0.0);
        assert!(ctrl.state.on_ground);
        assert_eq!(ctrl.state.velocity[1], 0.0);
        assert!((ctrl.state.position[1] - 0.9).abs() < 1e-5);
    }

    #[test]
    fn test_capsule_aabb() {
        let ctrl = new_character_controller(0.4, 1.8);
        let (mn, mx) = capsule_aabb(&ctrl);
        assert!(mn[0] < mx[0]);
        assert!(mn[1] < mx[1]);
        assert!(mn[2] < mx[2]);
        assert!((mx[0] - mn[0] - 0.8).abs() < 1e-5);
    }

    #[test]
    fn test_foot_head_position() {
        let ctrl = new_character_controller(0.4, 1.8);
        let foot = character_foot_position(&ctrl);
        let head = character_head_position(&ctrl);
        assert!((head[1] - foot[1] - 1.8).abs() < 1e-5);
    }

    #[test]
    fn test_can_step_up() {
        let ctrl = new_character_controller(0.4, 1.8);
        assert!(can_step_up(&ctrl, 0.1));
        assert!(!can_step_up(&ctrl, 1.0));
    }

    #[test]
    fn test_push_character() {
        let mut ctrl = new_character_controller(0.4, 1.8);
        ctrl.state.velocity = [0.0; 3];
        push_character(&mut ctrl, [1.0, 2.0, 3.0]);
        assert!((ctrl.state.velocity[0] - 1.0).abs() < 1e-5);
        assert!((ctrl.state.velocity[1] - 2.0).abs() < 1e-5);
        assert!((ctrl.state.velocity[2] - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_enable_disable_controller() {
        let mut ctrl = new_character_controller(0.4, 1.8);
        enable_controller(&mut ctrl);
        assert!(ctrl.enabled);
        disable_controller(&mut ctrl);
        assert!(!ctrl.enabled);
        enable_controller(&mut ctrl);
        assert!(ctrl.enabled);
    }

    #[test]
    fn test_character_speed() {
        let mut ctrl = new_character_controller(0.4, 1.8);
        ctrl.state.velocity = [3.0, 0.0, 4.0];
        let speed = character_speed(&ctrl);
        assert!((speed - 5.0).abs() < 1e-4);
    }
}
