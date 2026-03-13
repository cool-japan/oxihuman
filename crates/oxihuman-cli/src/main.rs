// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! OxiHuman CLI -- generate morphed human meshes and manage asset packs.

use anyhow::Result;

mod commands;
mod help;
mod utils;

fn main() {
    let all_args: Vec<String> = std::env::args().skip(1).collect();

    if all_args.is_empty() || all_args[0] == "--help" || all_args[0] == "-h" {
        help::print_help();
        return;
    }

    let subcommand = &all_args[0];
    let sub_args = &all_args[1..];

    let result: Result<()> = match subcommand.as_str() {
        "generate" => commands::generate::cmd_generate(sub_args),
        "pack-build" => commands::pack::cmd_pack_build(sub_args),
        "validate" => commands::info::cmd_validate(sub_args),
        "info" => commands::info::cmd_info(sub_args),
        "session" => commands::info::cmd_session(sub_args),
        "stats" => commands::info::parse_stats_args(sub_args),
        "proxies" => commands::misc::cmd_proxies(sub_args),
        "workspace" => {
            commands::info::cmd_workspace_info();
            Ok(())
        }
        "quantize" => commands::pack::cmd_quantize(sub_args),
        "morph-export" => commands::pack::cmd_morph_export(sub_args),
        "zip-pack" => commands::pack::cmd_zip_pack(sub_args),
        "stl" => commands::export::cmd_stl(sub_args),
        "collada" => commands::export::cmd_collada(sub_args),
        "gltf-sep" => commands::export::cmd_gltf_sep(sub_args),
        "svg" => commands::export::cmd_svg(sub_args),
        "lod-export" => commands::export::cmd_lod_export(sub_args),
        "variant-pack" => commands::export::cmd_variant_pack(sub_args),
        "report" => commands::export::cmd_report(sub_args),
        "asset-bundle" => commands::pack::cmd_asset_bundle(sub_args),
        "pc2" => commands::anim::cmd_pc2(sub_args),
        "mdd" => commands::anim::cmd_mdd(sub_args),
        "validate-pack" => commands::pack::cmd_validate_pack(sub_args),
        "target-info" => commands::info::cmd_target_info(sub_args),
        "sign-pack" => commands::pack::cmd_sign_pack(sub_args),
        "verify-sign" => commands::pack::cmd_verify_sign(sub_args),
        "batch-chars" => commands::misc::cmd_batch_chars(sub_args),
        "anim-bake" => commands::anim::cmd_anim_bake(sub_args),
        "stream-export" => commands::anim::cmd_stream_export(sub_args),
        "plugin-list" => {
            commands::info::cmd_plugin_list();
            Ok(())
        }
        "remesh" => commands::misc::cmd_remesh(sub_args),
        "physics-export" => commands::misc::cmd_physics_export(sub_args),
        "camera-info" => {
            commands::info::cmd_camera_info();
            Ok(())
        }
        "pack-dist-manifest" => commands::pack::cmd_pack_dist_manifest(sub_args),
        "pack-verify-dist" => commands::pack::cmd_pack_verify_dist(sub_args),
        "pack-wizard" => commands::wizard::cmd_pack_wizard(sub_args),
        other => {
            eprintln!("Unknown subcommand: {}", other);
            eprintln!("Run with --help for usage.");
            std::process::exit(1);
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}
