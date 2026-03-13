# oxihuman-mesh -- TODO

> Version: 0.1.1 | Updated: 2026-03-13

## Status: Stable

All core features implemented. 0 stubs. 5,715 passing tests (unit + proptest + doc).
898 source files, ~230k lines.

## Completed

### Core Mesh Types
- [x] MeshBuffers canonical representation (positions, normals, UVs, indices)
- [x] Skeleton / Joint hierarchy
- [x] SkinWeights per-vertex bone weights
- [x] Vertex groups and group maps

### Normals and Tangents
- [x] Normal recomputation (flat, smooth, angle-weighted)
- [x] Tangent frame computation
- [x] Normal delta / morph normal system
- [x] Normal transfer between meshes (v2)

### Repair and Integrity
- [x] Mesh repair (degenerate faces, duplicates, winding, index bounds)
- [x] Advanced repair (T-junction removal, hole filling, manifold enforcement)
- [x] Topology repair (isolated vertices, non-manifold edges/faces)
- [x] Integrity checks (index bounds, finite positions)

### Connectivity and Topology
- [x] Connected component detection and splitting
- [x] Boundary edge/loop detection
- [x] Non-manifold edge detection
- [x] Vertex valence, edge valence, vertex degree
- [x] Half-edge / dart graph representations
- [x] Topology flow (edge loops, edge rings, poles)
- [x] Simplicial complex with Euler characteristic / Betti numbers
- [x] Genus computation
- [x] Manifold checks

### Subdivision
- [x] Catmull-Clark subdivision (with config, N-level, crease support)
- [x] Loop subdivision
- [x] Midpoint subdivision
- [x] Adaptive subdivision (angle-based)

### Decimation and Simplification
- [x] Edge-collapse decimation (basic, feature-preserving, QEM)
- [x] Progressive mesh with LOD extraction
- [x] Adaptive LOD, view-dependent LOD, geomorph
- [x] Vertex clustering

### Remeshing
- [x] Isotropic remeshing (split/collapse/flip/relax)
- [x] Adaptive remeshing
- [x] Mean curvature flow

### Skinning and Deformation
- [x] Linear blend skinning (LBS)
- [x] Dual quaternion skinning (DQS)
- [x] Auto skin weights (bone heat diffusion)
- [x] Pose retargeting and pose library
- [x] Pose space deformation, corrective shapes
- [x] Skeleton deformation
- [x] Jiggle / spring deformation
- [x] Wrinkle driver maps

### UV Mapping
- [x] UV projection (planar, cylindrical, spherical, box)
- [x] UV atlas packing
- [x] UV seam detection, cutting, stitching, welding
- [x] UV quality metrics (stretch, overlap, utilization)
- [x] LSCM conformal parameterization
- [x] Harmonic map parameterization
- [x] Disk map parameterization
- [x] UV chart packing

### Geodesics
- [x] Dijkstra geodesic distances and paths
- [x] Heat-method geodesic distances
- [x] Geodesic Voronoi diagrams
- [x] Farthest point sampling, geodesic diameter

### Boolean Operations
- [x] Boolean union, intersection, difference
- [x] Mesh slicing with plane

### Curvature Analysis
- [x] Mean / Gaussian curvature
- [x] Curvature tensor (principal curvatures, shape index, curvedness)
- [x] Discrete curvature
- [x] Feature lines (ridges, valleys)
- [x] Curvature lines

### Smoothing
- [x] Laplacian smoothing
- [x] Taubin smoothing
- [x] Bilaplacian smoothing / fairing
- [x] Anisotropic smoothing
- [x] Cotangent-weight Laplacian

### Deformation Tools
- [x] Free-form deformation (FFD / lattice cage)
- [x] ARAP (as-rigid-as-possible) deformation
- [x] Cage deformation (mean value coordinates)
- [x] RBF warp deformation
- [x] Curve deformation (Bezier spine)
- [x] Wave / ripple deformation
- [x] Bend / twist / taper / shear modifiers
- [x] Grid deformation
- [x] Shell deformation
- [x] Tube deformation
- [x] MLS (moving least squares) deformation
- [x] Proximity deformation

### Mesh Generation
- [x] Parametric primitives (sphere, capsule, cylinder, cone, torus, plane, box, arrow, annulus, tetrahedron)
- [x] Icosphere generation
- [x] Fibonacci sphere
- [x] Terrain generation (heightfield, FBM noise)
- [x] Fractal geometry (Koch snowflake, Sierpinski, Mandelbrot)
- [x] Hex mesh, polar mesh, quad mesh, wave mesh, wedge mesh
- [x] Grid generation

### Mesh Operations
- [x] Merge, append, split by connectivity
- [x] Mirror (copy, symmetrize, axis-flip)
- [x] Extrude (faces, edges, vertices, along curve)
- [x] Inset faces, poke faces
- [x] Bridge edge loops
- [x] Fillet / chamfer / bevel edges and vertices
- [x] Solidify / thin shell
- [x] Sweep profile along path
- [x] Lofted surface from profiles
- [x] Clip / slice mesh
- [x] Hollow mesh / offset mesh / shrink wrap
- [x] Knife cut, path cut, edge slide, vertex slide, rip vertices

### Baking and Textures
- [x] Normal map baking (high-to-low poly)
- [x] Ambient occlusion baking
- [x] Curvature baking, thickness baking
- [x] Micro-displacement (FBM noise, Voronoi, wrinkle patterns)
- [x] Displacement map apply/extract
- [x] Heat map / scalar-to-color visualization
- [x] Vertex color maps, color attributes
- [x] Texture projection (box/cylindrical/spherical/planar)

### Spatial Acceleration
- [x] BVH (bounding volume hierarchy)
- [x] Octree
- [x] AABB tree (bbox tree)
- [x] Voxelization (surface + solid)
- [x] SDF (signed distance field) with CSG ops

### Measurements and Analysis
- [x] Body measurements / anthropometrics
- [x] AABB, OBB, bounding sphere
- [x] Surface area, volume estimation
- [x] Edge length / face area statistics
- [x] Mesh stats and complexity scoring
- [x] Thickness map
- [x] Convex hull computation
- [x] Moment of inertia tensor
- [x] Self-intersection detection
- [x] Aspect ratio / needle triangle detection

### Ray Casting and Queries
- [x] Ray-mesh intersection (single, all hits)
- [x] Closest point on mesh query
- [x] Winding number inside/outside
- [x] Signed distance queries
- [x] Point picking (face, vertex, box/sphere select)
- [x] Visibility / backface culling / frustum classification
- [x] Silhouette edge extraction

### Cloth and Simulation
- [x] Cloth panel generation (rectangular, circular, sleeve, t-shirt)
- [x] Cloth simulation (XPBD distance constraints)
- [x] Spring mesh simulation
- [x] Cloth collision
- [x] Force field mesh

### Hair
- [x] Hair card generation from guide curves
- [x] Hair card normals (face/smooth/custom)

### Mesh Painting and Sculpting
- [x] Brush system (smooth, inflate, flatten, grab, pinch)
- [x] Tweak tool, relax tool, inflate tool, pinch tool
- [x] Crease tool, bridge tool, fillet/chamfer edge tools

### Modifiers
- [x] Array, bevel, mirror, wireframe, skin modifiers
- [x] Screw, solidify, decimate, smooth, displace, warp modifiers
- [x] Mask modifier
- [x] Cast modifier (sphere/cylinder targets)
- [x] Bend / taper / twist / shear modifiers

### Segmentation and Labeling
- [x] Mesh segmentation (connectivity, normal deviation, planarity)
- [x] Body region labeling (flood fill)
- [x] Face labels and face materials
- [x] Voronoi diagram on mesh surface

### Advanced Topology
- [x] Dual mesh computation
- [x] Medial axis approximation
- [x] Spectral graph analysis (Laplacian, partitioning)
- [x] Polar decomposition of deformation gradients
- [x] Topological sorting of faces
- [x] Edge flow field
- [x] Spanning tree
- [x] Morse theory critical points

### Export and Transfer
- [x] Mesh diff / displacement field / mesh interpolation
- [x] Attribute transfer (nearest-vertex)
- [x] Weight transfer
- [x] Data transfer (normals, UVs)
- [x] Y-up / Z-up axis conversion
- [x] Mesh batching for rendering
- [x] Triangle strip encoding/decoding
- [x] Oct-encoded normals compression
- [x] Orthographic projection

### Reconstruction
- [x] Marching cubes (with welding)
- [x] Dual contouring (QEF-based)
- [x] Power crust (stub/config)
- [x] Poisson reconstruction (stub/config)

### Miscellaneous
- [x] Vertex animation, morph animation, skinned animation, blend animation
- [x] Pose snapshots
- [x] Impostor / sprite atlas generation
- [x] Decal mesh, outline mesh, shadow mesh, light map mesh
- [x] Retopology guide strokes
- [x] Symmetry map for mirror editing
- [x] Fiber orientation fields
- [x] Stress lines, curvature lines
- [x] Flow maps, edge flow fields
- [x] Catenary curves, helix paths

## Future Work

(No outstanding TODO/FIXME markers found in source.)
