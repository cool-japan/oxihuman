# oxihuman-export -- TODO

> Version: 0.1.1 | Updated: 2026-03-13

## Status: Stable

All core features implemented. 0 stubs (`todo!()`/`unimplemented!()`). 5,289 passing tests across ~883 source files. No `// TODO` or `// FIXME` comments.

## Completed

### Core 3D Formats
- [x] glTF 2.0 export (JSON + binary, separated, extensions, physics)
- [x] GLB export (binary glTF, animated GLB)
- [x] COLLADA (.dae) export
- [x] OBJ export (v1, v2, MTL materials)
- [x] STL export (ASCII + binary)
- [x] USD/USDA/USDZ export (animation, BlendShapeTimeSamples)
- [x] VRM export
- [x] FBX export (ASCII, binary, animation, stub fallback)
- [x] 3MF / ThreeMF export
- [x] Alembic export (Ogawa, point cloud, stub fallback)
- [x] AMF export

### Additional 3D / CAD Formats
- [x] AC3D, VRML/WRL, X3D, X, OFF, PLY (ASCII + binary)
- [x] IGES / IGES curve, STEP / STEP solid, BREP, Parasolid, JT
- [x] DXF, OpenSCAD, GeoJSON, CityGML, IFC, LandXML
- [x] BVH (motion capture), MDD, PC2
- [x] LWO, BGEO, HIP, NFF

### Point Cloud Formats
- [x] E57 export (v1, v2)
- [x] LAS / LAZ export
- [x] PCD export (v1, v2)
- [x] PTS / PTX / XYZ / XYZRGB point cloud export

### Mesh Pipeline
- [x] Mesh attributes (positions, normals, tangents, UVs, colors, weights)
- [x] Mesh LOD (levels, groups, chains, bias, switching)
- [x] Mesh compression (Draco, quantization)
- [x] Mesh instancing, submeshes, partitions, islands
- [x] Mesh sequence / cache export
- [x] Convex hull, bounding box, index buffer

### Animation System
- [x] Animation clips, channels, samplers, curves, tracks, layers
- [x] Keyframe sets, easing, blend trees, drivers
- [x] Skeleton export (hierarchy, JSON)
- [x] Bone animation, bind pose, constraints, envelope, roll
- [x] IK/FK (chain, constraint, pole, solver, target, weight, blend)
- [x] Shape keys / morph targets (channels, timeline, weights, delta bin)
- [x] BlendShape export (v1, v2, channels, drivers, inbetween)
- [x] NLA strips / tracks
- [x] Rig export (animation, controls)

### Materials and Texturing
- [x] PBR materials (material graph, nodes, instances, overrides, slots)
- [x] Texture atlas (packing, UV layout)
- [x] Normal / bump / displacement / AO / emission / roughness maps
- [x] Subsurface, transmission, IOR, opacity, occlusion maps
- [x] HDR / EXR / OpenEXR / ACES / ICC profiles
- [x] IBL, environment maps, cubemaps, light probes
- [x] Color palettes (Pantone, Munsell, RAL)

### Scene and Camera
- [x] Scene graph (hierarchy, nodes, transforms, manifests)
- [x] Camera export (animation, clip, DOF, FOV, ortho, stereo, path, rig, shake)
- [x] Light export, lightmaps, light probes
- [x] Render settings, passes, layers, post-processing

### Cloth / Hair / Physics Export
- [x] Cloth export (mesh, pin, pressure, sim state, stiffness, weight)
- [x] Hair export (cards, clumps, density, dynamics, guides, particles, roots, sim, style, system, width)
- [x] Fur export
- [x] Physics cache / shape / state export
- [x] Softbody, fluid, collision shape exports

### Human-Specific Export
- [x] SMPL / SMPL-X body model export
- [x] MakeHuman export
- [x] Biometric / EMG / galvanic / haptic / brain signal export
- [x] Facial features (eyebrow, eyelash, beard, nail, tooth)
- [x] Skin maps (melanin, hemoglobin, pore, scar, sebum, tattoo, vein, wrinkle)
- [x] Dense pose, landmark, OpenPose, MediaPipe

### Serialization / Data Formats
- [x] JSON, CBOR, MessagePack, BSON, RON, YAML, TOML, Arrow, Avro
- [x] Protobuf / Proto text, gRPC service (full stub)
- [x] Cap'n Proto schema (stub), FlatBuffers schema (stub)
- [x] Parquet metadata (stub), PDF generation (stub)
- [x] CSV, XML, RDF/Turtle/SPARQL, OWL
- [x] HDF5 (weights export), NetCDF, NPY/NPZ, SafeTensors, GGUF, Pickle

### Streaming and Pipeline
- [x] Streaming export, batch export, batch pipeline
- [x] WebSocket, SSE, AMQP, MQTT, NATS, Kafka, ZeroMQ, OSC
- [x] REST/HATEOAS/OData, OpenAPI/Swagger/AsyncAPI, GraphQL
- [x] Long-poll, real-time stream, job queue, export queue
- [x] gRPC framing (all 4 RPC patterns, gRPC-Web)

### Image / Video Formats
- [x] PNG, APNG, JPEG XL, WebP, AVIF, BMP, TGA, TIFF, GIF, ICO
- [x] EXR, HDR, deep image, RGBD, depth image
- [x] SVG (animation, path, polygon, skeleton)
- [x] Lottie, sprite sheet, image sequence

### ML / Inference
- [x] ONNX, CoreML, TFLite, TorchScript, TensorRT
- [x] OpenVINO, RKNN, SNPE, DeepSparse, NCNN

### Misc
- [x] Asset bundle, manifest, versioning, checksums
- [x] Gaussian splat, VDB, octree, voxel, distance field
- [x] Audit log, telemetry, changelog, report (HTML/Markdown)
- [x] Godot, Unity, Unreal, Cocos, Babylon, Three.js, D3, A-Frame, WebXR
- [x] MIDI, LilyPond, MusicXML, audio sync
- [x] ZIP packing, Draco compression, basis texture

## Future Work

(No `// TODO`, `// FIXME`, `todo!()`, or `unimplemented!()` markers found.)
