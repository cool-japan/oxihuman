# oxihuman-physics

Part of the [OxiHuman](../../README.md) workspace — privacy-first, client-side human body generator in pure Rust.

**Version:** 0.1.0 | **Status:** Partial | **Updated:** 2026-03-11

| Metric | Value |
|--------|-------|
| Passing tests | 4,973 |
| Public API items | 7,274 |
| Source files | 828 `.rs` files |
| Stub modules | 50+ (advanced biomechanics / actuators) |

---

## Overview

`oxihuman-physics` provides the full physics simulation stack for the OxiHuman ecosystem. Core collision detection, cloth, rigid body, and position-based dynamics solvers are production-ready. Biomechanics sensors/actuators and advanced material models are Alpha/Partial stubs reserved for future development.

No feature flags are required. All modules are compiled unconditionally; stub modules emit explicit stub markers (no `todo!()` or `unimplemented!()` macros are present).

---

## Dependency

```toml
[dependencies]
oxihuman-physics = "0.1"
```

### Workspace dependencies

```toml
[dependencies]
anyhow.workspace = true
serde_json.workspace = true
oxihuman-core.workspace = true
oxihuman-mesh.workspace = true
oxihuman-morph.workspace = true
```

---

## Module Reference

### Core Physics — Stable

| Module | Description |
|--------|-------------|
| `cloth` | Cloth simulation (mass-spring / constraint hybrid) |
| `cloth_pattern` | Cloth panel pattern cutting and stitching |
| `cloth_tear` | Tearing propagation model |
| `collision` | Broad- and narrow-phase collision detection |
| `contact_solver` | Iterative contact resolution |
| `contact_lcp` | Linear Complementarity Problem contact solver |
| `soft_body` | Soft body volumetric dynamics |
| `soft_body_mass_spring` | Mass-spring soft body variant |
| `rigid_body` | Rigid body integration and response |
| `dynamic_body` | Fully dynamic rigid body actor |
| `kinematic_body` | Kinematically driven rigid body actor |
| `character_controller` | Capsule-based character physics controller |
| `hair` | Hair strand dynamics simulation |
| `constraint` | Generic constraint definitions |
| `constraint_solver` | Constraint graph solver |
| `pbd_solver` | Position-Based Dynamics (PBD) solver |
| `xpbd_solver` | Extended Position-Based Dynamics (XPBD) solver |
| `rope_sim` | Inextensible rope simulation |
| `rope_cloth` | Rope–cloth coupling |
| `material` | Base material property definitions |

---

### Advanced Solvers — Beta

| Module | Description |
|--------|-------------|
| `fem_linear` | Linear Finite Element Method solver |
| `fem_corotational` | Co-rotational FEM for large deformations |
| `isogeometric_analysis` | IGA (NURBS-based FEM variant) |
| `boundary_element` | Boundary Element Method |
| `peridynamics` | Non-local elasticity / fracture model |

---

### Fluid Simulation — Beta

| Module | Description |
|--------|-------------|
| `sph_fluid` | Smoothed Particle Hydrodynamics — fluid |
| `sph_density` | SPH density estimation |
| `lattice_boltzmann` | Lattice Boltzmann Method (LBM) v1 |
| `lattice_boltzmann_v2` | LBM v2 with improved boundary conditions |
| `fluid_height` | Height-field shallow water |
| `fluid_2d` | 2D Eulerian fluid grid |
| `navier_stokes_2d` | 2D incompressible Navier–Stokes solver |
| `aerodynamics` | Aerodynamic force model |
| `wind_force` | Procedural wind force fields |
| `buoyancy` | Buoyancy and hydrostatic pressure |

---

### Material Models — 40+ types (Beta / Alpha)

Categories covered:

- **Hyperelastic** — Neo-Hookean, Mooney–Rivlin, Ogden, Yeoh
- **Viscoelastic** — Maxwell, Kelvin–Voigt, standard linear solid
- **Anisotropic** — Transverse isotropic, orthotopic fiber-reinforced
- **Composite** — Laminate, woven fiber, short-fiber random
- **Porous** — Biot consolidation, biphasic cartilage
- **Foam** — Open-cell, closed-cell, crushable foam
- **Auxetic** — Negative Poisson ratio lattice structures
- **Metamaterial** — Pentamode, acoustic, mechanical metamaterials
- **Phase-field fracture** — Brittle and ductile crack propagation
- **Plate bending** — Kirchhoff–Love and Mindlin–Reissner plates
- **Creep / Fatigue** — Norton–Bailey creep, S-N fatigue curves

---

### Biomechanics — Alpha / Partial Stubs

#### Sensors

| Module | Description |
|--------|-------------|
| `sensor_imu` | Inertial Measurement Unit simulation |
| `sensor_force_plate` | Ground reaction force plate |
| `sensor_emg` | Electromyography signal model |
| `sensor_mocap` | Motion capture marker set |

#### Actuators

| Module | Description |
|--------|-------------|
| `actuator_dc_motor` | DC motor torque / back-EMF model |
| `actuator_servo` | PWM servo position control |
| `actuator_hydraulic` | Hydraulic cylinder actuator |
| `actuator_cable_drive` | Bowden-cable tendon drive |
| `actuator_tendon_drive` | Direct tendon actuation |

#### Biological Models

| Module | Description |
|--------|-------------|
| `blood_pressure_sim` | Arterial blood pressure waveform |
| `lung_mechanics` | Lung compliance and airway resistance |
| `heart_valve_model` | Heart valve leaflet dynamics |
| `muscle_fatigue_model` | Muscle fatigue accumulation |
| `cartilage_model` | Articular cartilage biphasic model |
| `bone_deform` | Cortical/trabecular bone deformation |

#### Control Systems

| Module | Description |
|--------|-------------|
| `balance_control` | Center-of-mass balance controller |
| `gait_scheduler` | Gait phase scheduler |
| `locomotion_fsm` | Finite state machine locomotion controller |

---

### Complex Systems — Alpha

| Module | Description |
|--------|-------------|
| `boids_simulation` | Flocking / boids emergent behavior |
| `crowd_simulation` | Pedestrian crowd dynamics |
| `chaos_pendulum` | Double pendulum chaos model |
| `lorenz_system` | Lorenz attractor integration |
| `reaction_diffusion` | Gray–Scott reaction-diffusion |
| `cellular_automaton` | General cellular automaton engine |
| `debris_system` | Explosion / debris particle spawning and lifecycle |

---

## Stability Breakdown

| Category | Stability | Notes |
|----------|-----------|-------|
| Cloth / cloth tearing | Stable | Production-ready |
| Collision detection | Stable | Production-ready |
| Rigid / kinematic body | Stable | Production-ready |
| PBD / XPBD solvers | Stable | Production-ready |
| Soft body | Stable | Production-ready |
| Character controller | Stable | Production-ready |
| Hair dynamics | Stable | Production-ready |
| FEM / IGA / BEM | Beta | Functional, API may change |
| SPH / LBM fluids | Beta | Functional, API may change |
| 40+ material models | Beta / Alpha | Some models are stubs |
| Biomechanics sensors | Alpha / Partial | Stub implementations |
| Biomechanics actuators | Alpha / Partial | Stub implementations |
| Biological models | Alpha / Partial | Stub implementations |
| Control systems | Alpha | Stub implementations |
| Complex systems | Alpha | Experimental |

> **Note:** Stub modules use explicit stub markers. The codebase contains zero `todo!()` or `unimplemented!()` macro invocations.

---

## Feature Flags

This crate has **no feature flags**. All modules are unconditionally compiled.

---

## License

Apache-2.0 — Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
