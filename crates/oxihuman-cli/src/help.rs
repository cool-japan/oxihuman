// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! CLI help text printer.

use oxihuman_morph::presets::BodyPreset;
use oxihuman_morph::ExpressionPreset;

pub fn print_help() {
    println!(
        "OxiHuman CLI v{} — Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)",
        env!("CARGO_PKG_VERSION")
    );
    println!();
    println!("SUBCOMMANDS:");
    println!("  generate      Build a morphed .glb file");
    println!("  pack-build    Scan targets directory and build manifest");
    println!("  validate      Validate a single .target file or a pack manifest");
    println!("  info          Print mesh/GLB/manifest info");
    println!("  session       Print info about a saved session JSON");
    println!("  stats         Parse an OBJ file and print mesh statistics");
    println!("  workspace     Print workspace/build info (version, crates, features)");
    println!("  quantize      Quantize a base OBJ mesh into a compact QMSH binary");
    println!("  morph-export  Export morph targets directory to OXMD binary format");
    println!("  zip-pack      Pack base mesh + targets into a self-contained ZIP");
    println!("  stl           Export mesh to STL (ASCII or binary)");
    println!("  collada       Export mesh to COLLADA (.dae)");
    println!("  gltf-sep      Export mesh to glTF + separate .bin file");
    println!("  svg           Export mesh wireframe/UV to SVG");
    println!("  lod-export    Export LOD pack (multiple decimation levels)");
    println!("  variant-pack  Export multiple character variants as a named pack");
    println!("  report        Generate an HTML pipeline report");
    println!("  asset-bundle  Pack base mesh + targets into an OXB asset bundle");
    println!("  pc2           Bake a base mesh (optionally animated) to a PC2 point cache");
    println!("  mdd           Bake a base mesh (optionally animated) to an MDD point cache");
    println!("  anim-bake     Bake an animated params source to PC2/MDD");
    println!("  stream-export Stream-export mesh positions in chunks");
    println!("  plugin-list   List built-in plugin descriptors");
    println!(
        "  remesh              Remesh an OBJ via voxelization + Marching Cubes (--voxel-size)"
    );
    println!("  physics-export      Export physics scene (gltf-physics or openxr)");
    println!("  camera-info         Print default camera rig JSON");
    println!("  pack-dist-manifest  Generate SHA-256 distribution manifest for a pack directory");
    println!("  pack-verify-dist    Verify a pack directory against a distribution manifest");
    println!("  pack-wizard         Interactive wizard to build an .oxp asset pack step by step");
    println!("  pack-core           Curate MakeHuman CC0 assets into an OHPK v1 core/full pack");
    println!();
    println!("GENERATE OPTIONS:");
    println!("  --base <PATH>            Base .obj mesh file (required)");
    println!("  --targets <DIR>          Directory of .target morph files");
    println!("  --params <JSON|PATH>     Inline JSON or path to params JSON file");
    println!("  --preset <NAME>          Body preset (average/athletic/slender/heavy/tall/petite/senior/child)");
    println!(
        "  --expression <NAME>      Expression preset (neutral/smile/frown/surprised/angry/sad)"
    );
    println!("  --output <PATH>          Output .glb file (required)");
    println!("  --output-obj <PATH>      Also export as Wavefront .obj");
    println!("  --max-targets <N>        Limit targets loaded (default: all)");
    println!("  --strict                 Use strict policy profile");
    println!("  --measurements           Print body measurements JSON to stdout");
    println!("  --save-session <PATH>    Save current params as a session JSON");
    println!("  --load-session <PATH>    Load params from a session JSON file");
    println!("  --quantize <PATH>        Also export a quantized QMSH binary");
    println!();
    println!("SESSION OPTIONS:");
    println!("  <FILE>                   Path to session .json file");
    println!();
    println!("PACK-BUILD OPTIONS:");
    println!("  --targets <DIR>          Targets directory to scan (required)");
    println!("  --output <PATH>          Output manifest .toml file");
    println!("  --max-targets <N>        Limit files scanned");
    println!();
    println!("VALIDATE OPTIONS:");
    println!("  <FILE>                   Path to .target file");
    println!("  --pack <MANIFEST>        Path to pack manifest .toml file");
    println!();
    println!("INFO OPTIONS:");
    println!("  <FILE>                   Path to .glb, .obj, or .toml manifest");
    println!();
    println!("STATS OPTIONS:");
    println!("  <FILE>                   Path to Wavefront .obj file (required)");
    println!("  --full                   Show full statistics (aspect ratios, face area range)");
    println!("  --json                   Output as JSON instead of human-readable text");
    println!();
    println!("PRESETS:     {}", BodyPreset::all_names().join(", "));
    println!("EXPRESSIONS: {}", ExpressionPreset::all_names().join(", "));
    println!();
    println!("PROXIES OPTIONS:");
    println!("  --base <PATH>            Wavefront .obj mesh file (required)");
    println!("  --output <PATH>          Write JSON output to file (default: stdout)");
    println!("  --json                   Force JSON output (always on for this command)");
    println!();
    println!("QUANTIZE OPTIONS:");
    println!("  --base <PATH>            Base .obj mesh file (required)");
    println!("  --output <PATH>          Output .qmsh binary file (required)");
    println!("  --stats                  Print quantization statistics to stdout");
    println!();
    println!("MORPH-EXPORT OPTIONS:");
    println!("  --base <PATH>            Base .obj mesh file (required)");
    println!("  --targets <DIR>          Directory of .target morph files (required)");
    println!("  --output <PATH>          Output .oxmd binary file (required)");
    println!("  --max-targets <N>        Limit number of targets exported (default: all)");
    println!();
    println!("ZIP-PACK OPTIONS:");
    println!("  --base <PATH>            Base .obj mesh file (required)");
    println!("  --targets <DIR>          Directory of .target morph files (required)");
    println!("  --output <PATH>          Output .zip archive file (required)");
    println!();
    println!("STL OPTIONS:");
    println!("  --base <PATH>            Base .obj mesh file (required)");
    println!("  --output <PATH>          Output .stl file (required)");
    println!("  --binary                 Write binary STL (default: ASCII)");
    println!("  --targets <DIR>          Directory of .target morph files");
    println!("  --params <JSON|PATH>     Inline JSON or path to params JSON file");
    println!("  --preset <NAME>          Body preset name");
    println!();
    println!("COLLADA OPTIONS:");
    println!("  --base <PATH>            Base .obj mesh file (required)");
    println!("  --output <PATH>          Output .dae file (required)");
    println!("  --targets <DIR>          Directory of .target morph files");
    println!("  --params <JSON|PATH>     Inline JSON or path to params JSON file");
    println!("  --preset <NAME>          Body preset name");
    println!("  --author <STRING>        Author metadata string (default: OxiHuman)");
    println!();
    println!("GLTF-SEP OPTIONS:");
    println!("  --base <PATH>            Base .obj mesh file (required)");
    println!("  --output <PATH>          Output .gltf file (required)");
    println!("  --bin <PATH>             Separate .bin path (default: same stem + .bin)");
    println!("  --targets <DIR>          Directory of .target morph files");
    println!("  --params <JSON|PATH>     Inline JSON or path to params JSON file");
    println!();
    println!("SVG OPTIONS:");
    println!("  --base <PATH>            Base .obj mesh file (required)");
    println!("  --output <PATH>          Output .svg file (required)");
    println!("  --projection <front|side|top>  Projection axis (default: front)");
    println!("  --uv                     Export UV map instead of projected wireframe");
    println!("  --width <N>              Canvas width in pixels (default: 800)");
    println!("  --height <N>             Canvas height in pixels (default: 600)");
    println!();
    println!("LOD-EXPORT OPTIONS:");
    println!("  --base <PATH>            Base .obj mesh file (required)");
    println!("  --output-dir <DIR>       Output directory for LOD GLBs (required)");
    println!("  --targets <DIR>          Directory of .target morph files");
    println!("  --params <JSON|PATH>     Inline JSON or path to params JSON file");
    println!("  --preset <NAME>          Body preset name");
    println!("  --levels <N>             Number of LOD levels (default: 3, uses built-in)");
    println!();
    println!("VARIANT-PACK OPTIONS:");
    println!("  --params-list <JSON_FILE> JSON array of param objects (required)");
    println!("  --base <PATH>            Base .obj mesh file (required)");
    println!("  --targets <DIR>          Directory of .target morph files");
    println!("  --output-dir <DIR>       Output directory for the pack (required)");
    println!("  --pack-name <NAME>       Pack name (default: oxihuman_variants)");
    println!();
    println!("REPORT OPTIONS:");
    println!("  --base <PATH>            Base .obj mesh file (required)");
    println!("  --output <HTML>          Output .html report file (required)");
    println!("  --targets <DIR>          Directory of .target morph files (optional)");
    println!("  --pack <MANIFEST>        Path to manifest JSON file (optional)");
    println!("  --title <STRING>         Report title (default: OxiHuman Report)");
    println!();
    println!("ASSET-BUNDLE OPTIONS:");
    println!("  --base <PATH>            Base .obj mesh file (required)");
    println!("  --targets <DIR>          Directory of .target morph files (required)");
    println!("  --output <BUNDLE>        Output .oxb bundle file (required)");
    println!("  --manifest <TOML>        Optional manifest TOML file");
    println!();
    println!("PC2 OPTIONS:");
    println!("  --input <PATH>           Base .obj mesh file (required)");
    println!("  --output <PATH>          Output .pc2 file (required)");
    println!("  --targets <DIR>          Directory of .target morph files (optional)");
    println!(
        "  --anim <JSON>            Keyframed or snapshot params JSON (optional; see anim-bake)"
    );
    println!("  --frames <N>             Frame count when --anim doesn't fix it (default: 10)");
    println!("  --fps <F32>              Sample rate stored in the header (default: 24.0)");
    println!("  --start-time <F32>       Start time stored in the header (default: 0.0)");
    println!();
    println!("MDD OPTIONS:");
    println!("  --input <PATH>           Base .obj mesh file (required)");
    println!("  --output <PATH>          Output .mdd file (required)");
    println!("  --targets <DIR>          Directory of .target morph files (optional)");
    println!(
        "  --anim <JSON>            Keyframed or snapshot params JSON (optional; see anim-bake)"
    );
    println!("  --frames <N>             Frame count when --anim doesn't fix it (default: 10)");
    println!("  --fps <F32>              Playback rate; also drives per-frame timestamps (default: 24.0)");
    println!();
    println!("ANIM-BAKE OPTIONS:");
    println!("  --input <PATH>           Base .obj mesh file (required)");
    println!("  --params-json <JSON>     Params JSON: dense per-frame array, or");
    println!("                           {{\"keyframes\":[{{\"time\":f32,\"params\":{{...}}}}]}} (required)");
    println!("  --targets <DIR>          Directory of .target morph files (optional; needed for");
    println!("                           animated params to actually displace vertices)");
    println!("  --output <PATH>          Output cache file (required)");
    println!("  --format <pc2|mdd>       Output format (default: pc2)");
    println!("  --fps <F32>              Sample rate (default: 30.0)");
    println!("  --frames <N>             Frame count for keyframe sources (default: derived from duration)");
    println!();
    println!("PACK-CORE OPTIONS:");
    println!("  --upstream <DIR>         MakeHuman data tree (default: assets/upstream/makehuman)");
    println!("  --tier <core|full>       core = curated adult-only budget pack; full = everything (default: core)");
    println!(
        "  --out <FILE>             Output .ohpk file (default: assets/packs or dist per tier)"
    );
    println!("  --manifest <TOML>        Asset manifest reference (default: assets/alpha_pack/oxihuman_assets.toml)");
    println!("  --budget-gzip-bytes <N>  Core-tier on-disk byte budget (default: 2097152)");
    println!("  --report <MD>            Reconstruction-error report path (core tier)");
    println!();
    println!("PACK-DIST-MANIFEST OPTIONS:");
    println!("  --pack-dir <DIR>         Directory to scan (required)");
    println!();
    println!("PACK-VERIFY-DIST OPTIONS:");
    println!("  --manifest <FILE>        Distribution manifest JSON file (required)");
    println!("  --pack-dir <DIR>         Pack directory to verify (required)");
}
