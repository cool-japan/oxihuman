//! Muscle line-of-action deformer (bulge along muscle axis).

#[allow(dead_code)]
pub struct MuscleLine {
    pub name: String,
    pub origin: [f32; 3],
    pub insertion: [f32; 3],
    pub max_bulge: f32,
    pub falloff_radius: f32,
    pub contraction: f32,
}

#[allow(dead_code)]
pub struct MuscleDeformation {
    pub vertex_deltas: Vec<[f32; 3]>,
    pub influence_weights: Vec<f32>,
}

#[allow(dead_code)]
pub struct MuscleGroup {
    pub name: String,
    pub muscles: Vec<MuscleLine>,
}

#[allow(dead_code)]
pub fn new_muscle_line(
    name: &str,
    origin: [f32; 3],
    insertion: [f32; 3],
    max_bulge: f32,
    falloff: f32,
) -> MuscleLine {
    MuscleLine {
        name: name.to_string(),
        origin,
        insertion,
        max_bulge,
        falloff_radius: falloff,
        contraction: 0.0,
    }
}

#[allow(dead_code)]
pub fn muscle_direction(muscle: &MuscleLine) -> [f32; 3] {
    let dx = muscle.insertion[0] - muscle.origin[0];
    let dy = muscle.insertion[1] - muscle.origin[1];
    let dz = muscle.insertion[2] - muscle.origin[2];
    let len = (dx * dx + dy * dy + dz * dz).sqrt();
    if len > 1e-6 {
        [dx / len, dy / len, dz / len]
    } else {
        [0.0, 1.0, 0.0]
    }
}

#[allow(dead_code)]
pub fn muscle_length(muscle: &MuscleLine) -> f32 {
    let dx = muscle.insertion[0] - muscle.origin[0];
    let dy = muscle.insertion[1] - muscle.origin[1];
    let dz = muscle.insertion[2] - muscle.origin[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[allow(dead_code)]
pub fn point_to_line_distance(point: [f32; 3], line_start: [f32; 3], line_end: [f32; 3]) -> f32 {
    let dx = line_end[0] - line_start[0];
    let dy = line_end[1] - line_start[1];
    let dz = line_end[2] - line_start[2];
    let len_sq = dx * dx + dy * dy + dz * dz;

    if len_sq < 1e-12 {
        let ex = point[0] - line_start[0];
        let ey = point[1] - line_start[1];
        let ez = point[2] - line_start[2];
        return (ex * ex + ey * ey + ez * ez).sqrt();
    }

    let t = ((point[0] - line_start[0]) * dx
        + (point[1] - line_start[1]) * dy
        + (point[2] - line_start[2]) * dz)
        / len_sq;
    let t = t.clamp(0.0, 1.0);

    let proj_x = line_start[0] + t * dx - point[0];
    let proj_y = line_start[1] + t * dy - point[1];
    let proj_z = line_start[2] + t * dz - point[2];
    (proj_x * proj_x + proj_y * proj_y + proj_z * proj_z).sqrt()
}

#[allow(dead_code)]
pub fn muscle_influence_weight(muscle: &MuscleLine, pos: [f32; 3]) -> f32 {
    let dist = point_to_line_distance(pos, muscle.origin, muscle.insertion);
    if muscle.falloff_radius < 1e-6 {
        return 0.0;
    }
    let normalized = (dist / muscle.falloff_radius).clamp(0.0, 1.0);
    (1.0 - normalized).max(0.0)
}

#[allow(dead_code)]
pub fn compute_muscle_deformation(
    muscle: &MuscleLine,
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
) -> MuscleDeformation {
    let n = positions.len().min(normals.len());
    let dir = muscle_direction(muscle);
    let mut vertex_deltas = Vec::with_capacity(n);
    let mut influence_weights = Vec::with_capacity(n);

    for i in 0..n {
        let pos = positions[i];
        let weight = muscle_influence_weight(muscle, pos);
        let influence = weight * muscle.contraction * muscle.max_bulge;

        // Compute radial direction: perpendicular to muscle axis
        let to_point = [
            pos[0] - muscle.origin[0],
            pos[1] - muscle.origin[1],
            pos[2] - muscle.origin[2],
        ];
        let dot = to_point[0] * dir[0] + to_point[1] * dir[1] + to_point[2] * dir[2];
        let along = [dir[0] * dot, dir[1] * dot, dir[2] * dot];
        let radial = [
            to_point[0] - along[0],
            to_point[1] - along[1],
            to_point[2] - along[2],
        ];
        let radial_len =
            (radial[0] * radial[0] + radial[1] * radial[1] + radial[2] * radial[2]).sqrt();

        let delta = if radial_len > 1e-6 {
            [
                radial[0] / radial_len * influence,
                radial[1] / radial_len * influence,
                radial[2] / radial_len * influence,
            ]
        } else {
            let nrm = normals[i];
            [nrm[0] * influence, nrm[1] * influence, nrm[2] * influence]
        };

        vertex_deltas.push(delta);
        influence_weights.push(weight);
    }

    MuscleDeformation {
        vertex_deltas,
        influence_weights,
    }
}

#[allow(dead_code)]
pub fn apply_muscle_deformation(
    positions: &mut [[f32; 3]],
    deform: &MuscleDeformation,
    weight: f32,
) {
    let n = positions.len().min(deform.vertex_deltas.len());
    for (pos, delta) in positions[..n]
        .iter_mut()
        .zip(deform.vertex_deltas[..n].iter())
    {
        pos[0] += delta[0] * weight;
        pos[1] += delta[1] * weight;
        pos[2] += delta[2] * weight;
    }
}

#[allow(dead_code)]
pub fn contract_muscle(muscle: &mut MuscleLine, amount: f32) {
    muscle.contraction = amount.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn relax_muscle(muscle: &mut MuscleLine) {
    muscle.contraction = 0.0;
}

#[allow(dead_code)]
pub fn muscle_group_deformation(
    group: &MuscleGroup,
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
) -> Vec<MuscleDeformation> {
    group
        .muscles
        .iter()
        .map(|m| compute_muscle_deformation(m, positions, normals))
        .collect()
}

#[allow(dead_code)]
pub fn add_muscle_to_group(group: &mut MuscleGroup, muscle: MuscleLine) {
    group.muscles.push(muscle);
}

#[allow(dead_code)]
pub fn new_muscle_group(name: &str) -> MuscleGroup {
    MuscleGroup {
        name: name.to_string(),
        muscles: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn default_arm_muscles() -> MuscleGroup {
    let mut group = new_muscle_group("arm");

    // Bicep: shoulder to elbow front
    let mut bicep = new_muscle_line("bicep", [0.15, 0.4, 0.0], [0.15, -0.1, 0.05], 0.02, 0.08);
    bicep.contraction = 0.0;

    // Tricep: shoulder to elbow back
    let mut tricep = new_muscle_line(
        "tricep",
        [0.15, 0.35, -0.02],
        [0.15, -0.1, -0.04],
        0.018,
        0.07,
    );
    tricep.contraction = 0.0;

    // Deltoid: acromion to humerus
    let mut deltoid = new_muscle_line("deltoid", [0.08, 0.42, 0.0], [0.18, 0.25, 0.0], 0.025, 0.1);
    deltoid.contraction = 0.0;

    group.muscles.push(bicep);
    group.muscles.push(tricep);
    group.muscles.push(deltoid);

    group
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_muscle_direction_unit_length() {
        let m = new_muscle_line("test", [0.0, 0.0, 0.0], [3.0, 4.0, 0.0], 0.01, 0.1);
        let dir = muscle_direction(&m);
        let len = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_muscle_direction_components() {
        let m = new_muscle_line("test", [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 0.01, 0.1);
        let dir = muscle_direction(&m);
        assert!((dir[0] - 1.0).abs() < 1e-5);
        assert!(dir[1].abs() < 1e-5);
        assert!(dir[2].abs() < 1e-5);
    }

    #[test]
    fn test_muscle_length() {
        let m = new_muscle_line("test", [0.0, 0.0, 0.0], [3.0, 4.0, 0.0], 0.01, 0.1);
        assert!((muscle_length(&m) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_muscle_length_zero() {
        let m = new_muscle_line("test", [1.0, 2.0, 3.0], [1.0, 2.0, 3.0], 0.01, 0.1);
        assert!(muscle_length(&m) < 1e-5);
    }

    #[test]
    fn test_point_to_line_distance_on_line() {
        // Point on the line should have distance ~0
        let dist = point_to_line_distance([0.5, 0.0, 0.0], [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert!(dist < 1e-5);
    }

    #[test]
    fn test_point_to_line_distance_perpendicular() {
        // Point 1 unit above midpoint of x-axis
        let dist = point_to_line_distance([0.5, 1.0, 0.0], [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert!((dist - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_point_to_line_distance_clamped() {
        // Point beyond the end of the segment
        let dist = point_to_line_distance([2.0, 1.0, 0.0], [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        // Closest point is [1,0,0], distance = sqrt(1+1) = sqrt(2)
        assert!((dist - 2.0f32.sqrt()).abs() < 1e-4);
    }

    #[test]
    fn test_muscle_influence_weight_on_axis() {
        let m = new_muscle_line("test", [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 0.01, 0.5);
        // Point on the axis midpoint, distance = 0, weight = 1
        let w = muscle_influence_weight(&m, [0.5, 0.0, 0.0]);
        assert!((w - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_muscle_influence_weight_far() {
        let m = new_muscle_line("test", [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 0.01, 0.5);
        // Point far away should have weight ~0
        let w = muscle_influence_weight(&m, [0.5, 10.0, 0.0]);
        assert!(w < 1e-5);
    }

    #[test]
    fn test_compute_muscle_deformation() {
        let mut m = new_muscle_line("test", [0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 0.05, 0.2);
        contract_muscle(&mut m, 1.0);
        let positions = vec![[0.1, 0.5, 0.0], [5.0, 5.0, 5.0]];
        let normals = vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let deform = compute_muscle_deformation(&m, &positions, &normals);
        assert_eq!(deform.vertex_deltas.len(), 2);
        assert_eq!(deform.influence_weights.len(), 2);
        // First vertex is close so it should have positive weight
        assert!(deform.influence_weights[0] > 0.0);
    }

    #[test]
    fn test_contract_relax() {
        let mut m = new_muscle_line("test", [0.0; 3], [1.0, 0.0, 0.0], 0.01, 0.1);
        contract_muscle(&mut m, 0.7);
        assert!((m.contraction - 0.7).abs() < 1e-5);
        relax_muscle(&mut m);
        assert!(m.contraction < 1e-5);
    }

    #[test]
    fn test_contract_clamp() {
        let mut m = new_muscle_line("test", [0.0; 3], [1.0, 0.0, 0.0], 0.01, 0.1);
        contract_muscle(&mut m, 2.0);
        assert!((m.contraction - 1.0).abs() < 1e-5);
        contract_muscle(&mut m, -1.0);
        assert!(m.contraction < 1e-5);
    }

    #[test]
    fn test_muscle_group() {
        let mut group = new_muscle_group("legs");
        assert!(group.muscles.is_empty());
        let m = new_muscle_line("quad", [0.0; 3], [0.0, -0.5, 0.0], 0.02, 0.1);
        add_muscle_to_group(&mut group, m);
        assert_eq!(group.muscles.len(), 1);
    }

    #[test]
    fn test_default_arm_muscles_has_three() {
        let group = default_arm_muscles();
        assert_eq!(group.muscles.len(), 3);
    }

    #[test]
    fn test_default_arm_muscles_names() {
        let group = default_arm_muscles();
        let names: Vec<&str> = group.muscles.iter().map(|m| m.name.as_str()).collect();
        assert!(names.contains(&"bicep"));
        assert!(names.contains(&"tricep"));
        assert!(names.contains(&"deltoid"));
    }

    #[test]
    fn test_muscle_group_deformation() {
        let group = default_arm_muscles();
        let positions = vec![[0.15f32, 0.3, 0.0]];
        let normals = vec![[0.0f32, 0.0, 1.0]];
        let deforms = muscle_group_deformation(&group, &positions, &normals);
        assert_eq!(deforms.len(), 3);
    }

    #[test]
    fn test_apply_muscle_deformation() {
        let mut m = new_muscle_line("test", [0.0; 3], [0.0, 1.0, 0.0], 0.1, 0.5);
        contract_muscle(&mut m, 1.0);
        let positions_orig = vec![[0.1f32, 0.5, 0.0]];
        let normals = vec![[1.0f32, 0.0, 0.0]];
        let deform = compute_muscle_deformation(&m, &positions_orig, &normals);
        let mut positions = positions_orig.clone();
        apply_muscle_deformation(&mut positions, &deform, 1.0);
        assert_eq!(positions.len(), 1);
    }
}
