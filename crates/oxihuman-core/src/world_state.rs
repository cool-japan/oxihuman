//! World/scene state manager for simulation coordination.

#[allow(dead_code)]
pub struct WorldStateConfig {
    pub fixed_delta_time: f32,
    pub max_entities: usize,
    pub gravity: [f32; 3],
    pub paused_on_start: bool,
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SimPhase {
    Init,
    Running,
    Paused,
    Done,
}

#[allow(dead_code)]
pub struct SystemEntry {
    pub name: String,
    pub priority: i32,
    pub enabled: bool,
    pub tick_count: u64,
}

#[allow(dead_code)]
pub struct WorldState {
    pub config: WorldStateConfig,
    pub phase: SimPhase,
    pub time: f64,
    pub frame: u64,
    pub delta_time: f32,
    pub entity_count: usize,
    pub systems: Vec<SystemEntry>,
}

#[allow(dead_code)]
pub fn default_world_state_config() -> WorldStateConfig {
    WorldStateConfig {
        fixed_delta_time: 1.0 / 60.0,
        max_entities: 10_000,
        gravity: [0.0, -9.81, 0.0],
        paused_on_start: false,
    }
}

#[allow(dead_code)]
pub fn new_world_state(config: WorldStateConfig) -> WorldState {
    let phase = if config.paused_on_start {
        SimPhase::Paused
    } else {
        SimPhase::Init
    };
    WorldState {
        config,
        phase,
        time: 0.0,
        frame: 0,
        delta_time: 1.0 / 60.0,
        entity_count: 0,
        systems: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn advance_world(world: &mut WorldState) {
    if world.phase == SimPhase::Running {
        world.time += world.delta_time as f64;
        world.frame += 1;
        tick_systems(world);
    }
}

#[allow(dead_code)]
pub fn world_time(world: &WorldState) -> f64 {
    world.time
}

#[allow(dead_code)]
pub fn world_frame(world: &WorldState) -> u64 {
    world.frame
}

#[allow(dead_code)]
pub fn set_sim_phase(world: &mut WorldState, phase: SimPhase) {
    world.phase = phase;
}

#[allow(dead_code)]
pub fn sim_phase(world: &WorldState) -> SimPhase {
    world.phase
}

#[allow(dead_code)]
pub fn world_delta_time(world: &WorldState) -> f32 {
    world.delta_time
}

#[allow(dead_code)]
pub fn register_system(world: &mut WorldState, name: &str, priority: i32) {
    // Check if already registered
    if world.systems.iter().any(|s| s.name == name) {
        return;
    }
    world.systems.push(SystemEntry {
        name: name.to_string(),
        priority,
        enabled: true,
        tick_count: 0,
    });
    // Sort by priority (higher = earlier)
    world.systems.sort_by(|a, b| b.priority.cmp(&a.priority));
}

#[allow(dead_code)]
pub fn tick_systems(world: &mut WorldState) {
    for sys in &mut world.systems {
        if sys.enabled {
            sys.tick_count += 1;
        }
    }
}

#[allow(dead_code)]
pub fn world_to_json(world: &WorldState) -> String {
    let mut parts = Vec::new();
    parts.push(format!("\"time\":{:.6}", world.time));
    parts.push(format!("\"frame\":{}", world.frame));
    parts.push(format!("\"delta_time\":{:.6}", world.delta_time));
    parts.push(format!("\"entity_count\":{}", world.entity_count));
    parts.push(format!("\"phase\":\"{}\"", sim_phase_name(world)));
    parts.push(format!("\"system_count\":{}", world.systems.len()));
    format!("{{{}}}", parts.join(","))
}

#[allow(dead_code)]
pub fn reset_world(world: &mut WorldState) {
    world.time = 0.0;
    world.frame = 0;
    world.entity_count = 0;
    world.phase = SimPhase::Init;
    for sys in &mut world.systems {
        sys.tick_count = 0;
    }
}

#[allow(dead_code)]
pub fn world_entity_count(world: &WorldState) -> usize {
    world.entity_count
}

#[allow(dead_code)]
pub fn sim_phase_name(world: &WorldState) -> &'static str {
    match world.phase {
        SimPhase::Init => "init",
        SimPhase::Running => "running",
        SimPhase::Paused => "paused",
        SimPhase::Done => "done",
    }
}

#[allow(dead_code)]
pub fn pause_world(world: &mut WorldState) {
    if world.phase == SimPhase::Running {
        world.phase = SimPhase::Paused;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_world() -> WorldState {
        new_world_state(default_world_state_config())
    }

    #[test]
    fn test_new_world_state_defaults() {
        let w = make_world();
        assert_eq!(world_frame(&w), 0);
        assert!((world_time(&w)).abs() < 1e-10);
        assert_eq!(sim_phase(&w), SimPhase::Init);
    }

    #[test]
    fn test_set_sim_phase() {
        let mut w = make_world();
        set_sim_phase(&mut w, SimPhase::Running);
        assert_eq!(sim_phase(&w), SimPhase::Running);
    }

    #[test]
    fn test_advance_world_increments_frame() {
        let mut w = make_world();
        set_sim_phase(&mut w, SimPhase::Running);
        advance_world(&mut w);
        assert_eq!(world_frame(&w), 1);
    }

    #[test]
    fn test_advance_world_increments_time() {
        let mut w = make_world();
        set_sim_phase(&mut w, SimPhase::Running);
        let dt = world_delta_time(&w);
        advance_world(&mut w);
        assert!((world_time(&w) - dt as f64).abs() < 1e-8);
    }

    #[test]
    fn test_advance_paused_world_no_change() {
        let mut w = make_world();
        set_sim_phase(&mut w, SimPhase::Paused);
        advance_world(&mut w);
        assert_eq!(world_frame(&w), 0);
        assert!((world_time(&w)).abs() < 1e-10);
    }

    #[test]
    fn test_pause_world_from_running() {
        let mut w = make_world();
        set_sim_phase(&mut w, SimPhase::Running);
        pause_world(&mut w);
        assert_eq!(sim_phase(&w), SimPhase::Paused);
    }

    #[test]
    fn test_pause_world_from_init_no_change() {
        let mut w = make_world();
        pause_world(&mut w);
        assert_eq!(sim_phase(&w), SimPhase::Init);
    }

    #[test]
    fn test_register_system() {
        let mut w = make_world();
        register_system(&mut w, "physics", 10);
        assert_eq!(w.systems.len(), 1);
    }

    #[test]
    fn test_register_system_no_duplicate() {
        let mut w = make_world();
        register_system(&mut w, "render", 5);
        register_system(&mut w, "render", 5);
        assert_eq!(w.systems.len(), 1);
    }

    #[test]
    fn test_register_system_sorted_by_priority() {
        let mut w = make_world();
        register_system(&mut w, "low", 1);
        register_system(&mut w, "high", 100);
        register_system(&mut w, "mid", 50);
        assert_eq!(w.systems[0].name, "high");
        assert_eq!(w.systems[2].name, "low");
    }

    #[test]
    fn test_tick_systems_increments_tick_count() {
        let mut w = make_world();
        register_system(&mut w, "anim", 10);
        tick_systems(&mut w);
        assert_eq!(w.systems[0].tick_count, 1);
    }

    #[test]
    fn test_world_to_json() {
        let w = make_world();
        let json = world_to_json(&w);
        assert!(json.contains("time"));
        assert!(json.contains("frame"));
        assert!(json.contains("phase"));
        assert!(json.contains("system_count"));
    }

    #[test]
    fn test_reset_world() {
        let mut w = make_world();
        set_sim_phase(&mut w, SimPhase::Running);
        advance_world(&mut w);
        advance_world(&mut w);
        reset_world(&mut w);
        assert_eq!(world_frame(&w), 0);
        assert!((world_time(&w)).abs() < 1e-10);
        assert_eq!(sim_phase(&w), SimPhase::Init);
    }

    #[test]
    fn test_world_entity_count() {
        let mut w = make_world();
        w.entity_count = 42;
        assert_eq!(world_entity_count(&w), 42);
    }

    #[test]
    fn test_sim_phase_name_all_variants() {
        let mut w = make_world();
        assert_eq!(sim_phase_name(&w), "init");
        set_sim_phase(&mut w, SimPhase::Running);
        assert_eq!(sim_phase_name(&w), "running");
        set_sim_phase(&mut w, SimPhase::Paused);
        assert_eq!(sim_phase_name(&w), "paused");
        set_sim_phase(&mut w, SimPhase::Done);
        assert_eq!(sim_phase_name(&w), "done");
    }

    #[test]
    fn test_paused_on_start_config() {
        let cfg = WorldStateConfig {
            fixed_delta_time: 1.0 / 60.0,
            max_entities: 100,
            gravity: [0.0, -9.81, 0.0],
            paused_on_start: true,
        };
        let w = new_world_state(cfg);
        assert_eq!(sim_phase(&w), SimPhase::Paused);
    }

    #[test]
    fn test_advance_multiple_frames() {
        let mut w = make_world();
        set_sim_phase(&mut w, SimPhase::Running);
        for _ in 0..10 {
            advance_world(&mut w);
        }
        assert_eq!(world_frame(&w), 10);
    }

    #[test]
    fn test_world_delta_time() {
        let w = make_world();
        let dt = world_delta_time(&w);
        assert!((dt - 1.0 / 60.0).abs() < 1e-5);
    }
}
