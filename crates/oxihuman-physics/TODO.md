# oxihuman-physics -- TODO

> Version: 0.1.2 | Updated: 2026-05-05

## Status: Stable

All core features implemented. 0 stubs (`todo!()`/`unimplemented!()`). 5,217 passing tests across 864 source files (~190k SLoC). No `// TODO` or `// FIXME` comments.

## Completed

### Rigid Body Dynamics
- [x] Rigid body (2D, definition, group, state, tree, compound, stack, transform)
- [x] Static body, kinematic body (scripted, controller, target)
- [x] Dynamic body, floating base, articulated body, articulation tree
- [x] Multibody dynamics, inverse dynamics, forward kinematics

### Soft Body Simulation
- [x] Soft body (2D, FEM, mass-spring, volume)
- [x] Deformable body / mesh / solid / spring
- [x] Membrane body / model / tension
- [x] Foam body / material / model / spring
- [x] Gel body / model, balloon body, rubber (auxetic, hyperelastic)

### Cloth Simulation
- [x] Cloth (2D, constraint, grid, patch, pattern, sim, tear, tearing, wind)
- [x] XPBD cloth, FEM cloth
- [x] Garment fit (v1, v2)

### FEM (Finite Element Method)
- [x] FEM linear (v1, v2), corotational, tetra
- [x] FEM cloth, soft body FEM
- [x] Isogeometric analysis, finite element 3D
- [x] Beam element, shell kinematics, plate bending

### SPH and Fluid
- [x] SPH fluid (density, pressure, viscosity v1/v2, surface tension, vorticity, boundary, multiphase)
- [x] Fluid (2D, advect, body, Eulerian, grid, height, lattice, particle, pressure, surface)
- [x] Smoothed particle (2D), meshfree SPH
- [x] Position-based fluids
- [x] Navier-Stokes 2D, shallow water, Poisson solver
- [x] Lattice Boltzmann (standard, D2Q9, v2), lattice gas v2

### Collision Detection
- [x] Broad phase (grid, hash, SAP, sort, pairs)
- [x] Narrow phase (GJK, EPA, GJK+EPA combined, MPR, SAT)
- [x] Capsule (body, collider, contact, pair, ray, sweep)
- [x] Sphere body, cone body, cylinder body, plane (body, collider)
- [x] Mesh collider, voxel collider
- [x] BVH tree, spatial hash, sweep cast, shape cast
- [x] Self-collision, self-intersection, CCD solver
- [x] Collision (2D, cache, detection, event, filter, geometry, layers, matrix, normal, pairs, response, stats)
- [x] Time of impact, penetration depth, continuous collision

### Contact and Friction
- [x] Contact (cache, estimator, friction model, graph, island, LCP, manifold v1/v2, material, mechanics, normal, pair, patch, point pool, pool, queue, reduction, resolver, solver PGS/seq, stiffness, store, velocity, warmstart)
- [x] Friction (coefficient, cone v1/v2, joint, model, patch, surface, surface model, Coulomb, rolling)

### Joints and Constraints
- [x] Joint types: ball, hinge, prismatic, slider, fixed, weld, universal, revolute, distance, gear, pulley, cone, pivot, screw, yoke
- [x] Joint features (2D, anchor, angle, chain, constraint, contact model, damper, drive, graph, limits, motor v1/v2/drive, spring, torque)
- [x] Constraint system (batch, bound, chain, distance, graph, group, hinge, island, Jacobian, limit, motor, point, row, slider, solver, solver PGS, spring, warm)
- [x] Position-based dynamics (PBD solver, XPBD solver/v2, distance, particle, shape, volume)
- [x] Projective dynamics, augmented Lagrangian, sequential impulse, VBD solver

### Integrators
- [x] Velocity Verlet, Verlet integrator, leapfrog integrator
- [x] Symplectic Euler, Runge-Kutta
- [x] Position integration, time stepping, sim step

### Hair Dynamics
- [x] Hair dynamics, hair simulation

### Muscle and Biomechanics
- [x] Muscle (activation, body, fatigue model, spring)
- [x] Tendon (model, viscoelastic)
- [x] Ligament spring, fascia model, cartilage (model, stress)
- [x] Bone (deform, remodeling)
- [x] Biomechanical loading, ergonomics model
- [x] Gait scheduler, locomotion FSM, step pattern, foot placement
- [x] Balance control / controller, posture sway model, push recovery
- [x] Fall detection, fall risk model, capture point, zero moment point, COM trajectory

### Physiological Systems
- [x] Cardiac output, coronary flow, pulmonary flow, venous return, blood pressure/viscosity
- [x] Lung mechanics (v1, v2), diaphragm model, rib cage model
- [x] Liver clearance, renal filtration/tubular reabsorption, gastric acid, intestinal absorption
- [x] Enzyme kinetics, reaction kinetics
- [x] Synovial fluid, interstitial fluid, lymph flow/node, osmotic pressure, capillary action
- [x] Eye pressure, intervertebral disc, spinal column model, bladder model
- [x] Neural signal model, reflex arc, vestibular model, proprioception (stub)
- [x] Circadian rhythm, sleep-wake cycle, thermoregulation core, melanin distribution
- [x] Sweat gland model, pain threshold model, wound healing model
- [x] Cell migration, DNA model, tumor growth (stub), SIR epidemic model
- [x] Digestive peristalsis, spleen model

### Sensors and Actuators
- [x] Sensors: IMU, EMG, force plate, load cell, strain gauge, tactile, pressure mat, flex, potentiometer, encoder, temperature, ultrasonic, depth camera, motion capture
- [x] Sensor stubs: camera, lidar
- [x] Actuators: DC motor, stepper, servo, hydraulic, pneumatic, linear motor, tendon drive, cable drive, ball screw, rack pinion, harmonic drive, worm gear, gear train, differential, soft robot, parallel robot

### Forces and Fields
- [x] Gravity (body, field, model, source, well 2D/3D, zone, n-body)
- [x] Drag (coefficient, force, model), air resistance
- [x] Buoyancy (2D, body, force, grid, sim), buoyant body
- [x] Wind (body, field, force, turbulence)
- [x] Magnetic (2D, field, particle, torque)
- [x] Electrostatic force, piezoelectric, electrostriction, ferroelectric, magnetostrictive
- [x] Thermal (convection, expansion, model, sim, thermoelastic stress)
- [x] Radiation (heat, transfer), tidal force
- [x] Vortex (field, model), vorticity confinement, turbulence force
- [x] Surface tension, adhesion (force, model), lubrication (force, model)

### Material Models
- [x] Plasticity (model), hyperelastic (model), viscoelastic (model)
- [x] Anisotropic (material, spring), ortho elastic
- [x] Composite material, fiber (body v1/v2, composite, network)
- [x] Granular (body, flow, material, sim)
- [x] Brittle body, ductile body, ice body, sand body, snow sim, powder body
- [x] Shape memory (alloy, effect), metamaterial (stub)
- [x] Porous (flow, media), foam, gel

### Fracture and Damage
- [x] Fracture (model, mechanics, mechanics props)
- [x] Crack propagation, discrete fracture, phase field fracture
- [x] Damage model, fatigue (life, model), creep (deform, model)
- [x] Buckling analysis, delamination model, peridynamics

### Nonlinear Dynamics / Chaos
- [x] Lorenz (attractor, system), Rossler attractor
- [x] Double pendulum, Duffing (body, oscillator), Van der Pol (oscillator)
- [x] Chaos pendulum, coupled oscillator, spring pendulum
- [x] Logistic map, Henon map, Julia orbit, Mandelbrot orbit
- [x] Turing pattern, reaction diffusion, Lotka-Volterra
- [x] Lyapunov exponent, bifurcation map, phase space, fractal dimension

### Misc Simulation
- [x] Ragdoll (config, physics)
- [x] Rope (2D, cloth, physics, segment, sim)
- [x] Crowd simulation, boids simulation, swarm behavior
- [x] Projectile (2D, body, motion v2), ballistic
- [x] Car physics 2D, conveyor belt, water wheel, windmill body
- [x] Explosion (2D, impulse), debris system
- [x] Ocean waves, wave (body, equation 1D, propagation)
- [x] Terrain (estimator, physics), portal physics 2D
- [x] Character controller, scene physics, scene query

### Infrastructure
- [x] Physics world (query), solver config, iteration config, stats, profiler
- [x] Body (AABB, activation, CCD, collision group, contact list, drag, dynamics, flag, force field, friction, group, inertia, mass override, material, pair, pair filter, pose, registry, sleep/sleeping, state snapshot, transform, velocity)
- [x] Island manager, sim island, warm start (v1, v2, warming)
- [x] Energy tracker/util, mass (distribution, properties, solver), moment of inertia
- [x] Impulse (accumulator, cache v1/v2, joint, pair, resolver, response, solver)
- [x] Shape (bounds, cast, distance, fitting, intersection, matching, volume)
- [x] SDF gen, distance field, level set advect, marching squares

## Future Work

(No `// TODO`, `// FIXME`, `todo!()`, or `unimplemented!()` markers found.)
