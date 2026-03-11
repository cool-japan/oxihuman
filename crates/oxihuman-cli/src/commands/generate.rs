// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The `generate` subcommand: build a morphed GLB from base mesh + targets + params.

use anyhow::{bail, Context, Result};
use std::path::PathBuf;

use oxihuman_core::policy::{Policy, PolicyProfile};
use oxihuman_export::obj::export_obj;
use oxihuman_export::params_json::export_mesh_measurements;
use oxihuman_export::pipeline::{run_pipeline, PipelineConfig};
use oxihuman_export::{quantize_mesh, write_quantized_bin};
use oxihuman_morph::params::ParamState;
use oxihuman_morph::presets::BodyPreset;
use oxihuman_morph::session::MorphSession;
use oxihuman_morph::ExpressionPreset;

use crate::utils::{build_mesh_with_expression, load_params};

#[allow(clippy::too_many_arguments)]
pub fn cmd_generate(args: &[String]) -> Result<()> {
    let mut base: Option<PathBuf> = None;
    let mut targets: Option<PathBuf> = None;
    let mut params_src: Option<String> = None;
    let mut preset_name: Option<String> = None;
    let mut expression_name: Option<String> = None;
    let mut output: Option<PathBuf> = None;
    let mut output_obj: Option<PathBuf> = None;
    let mut max_targets: Option<usize> = None;
    let mut strict = false;
    let mut measurements = false;
    let mut save_session: Option<PathBuf> = None;
    let mut load_session: Option<PathBuf> = None;
    let mut quantize_output: Option<PathBuf> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--base" => {
                i += 1;
                base = Some(PathBuf::from(&args[i]));
            }
            "--targets" => {
                i += 1;
                targets = Some(PathBuf::from(&args[i]));
            }
            "--params" => {
                i += 1;
                params_src = Some(args[i].clone());
            }
            "--preset" => {
                i += 1;
                preset_name = Some(args[i].clone());
            }
            "--expression" => {
                i += 1;
                expression_name = Some(args[i].clone());
            }
            "--output" => {
                i += 1;
                output = Some(PathBuf::from(&args[i]));
            }
            "--output-obj" => {
                i += 1;
                output_obj = Some(PathBuf::from(&args[i]));
            }
            "--max-targets" => {
                i += 1;
                max_targets = Some(args[i].parse()?);
            }
            "--strict" => {
                strict = true;
            }
            "--measurements" => {
                measurements = true;
            }
            "--save-session" => {
                i += 1;
                save_session = Some(PathBuf::from(&args[i]));
            }
            "--load-session" => {
                i += 1;
                load_session = Some(PathBuf::from(&args[i]));
            }
            "--quantize" => {
                i += 1;
                quantize_output = Some(PathBuf::from(&args[i]));
            }
            other => bail!("unknown option: {}", other),
        }
        i += 1;
    }

    let base = base.context("--base is required for generate")?;
    let output = output.context("--output is required for generate")?;

    if !base.exists() {
        bail!("base mesh not found: {}", base.display());
    }

    let policy = if strict {
        Policy::new(PolicyProfile::Strict)
    } else {
        Policy::new(PolicyProfile::Standard)
    };

    // Params priority: --load-session > --preset > --params > default
    let params = if let Some(ref session_path) = load_session {
        if !session_path.exists() {
            bail!("session file not found: {}", session_path.display());
        }
        let session = MorphSession::load(session_path)
            .with_context(|| format!("loading session from {}", session_path.display()))?;
        eprintln!("OxiHuman: loaded session from {}", session_path.display());
        if let Some(ref label) = session.label {
            eprintln!("OxiHuman: session label: {}", label);
        }
        session.to_param_state()
    } else if let Some(name) = &preset_name {
        BodyPreset::from_name(name)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "unknown preset '{}'. Available: {}",
                    name,
                    BodyPreset::all_names().join(", ")
                )
            })?
            .params()
    } else if let Some(src) = &params_src {
        load_params(src)?
    } else {
        ParamState::default()
    };

    eprintln!("OxiHuman: base={}", base.display());
    eprintln!(
        "OxiHuman: h={:.2} w={:.2} m={:.2} a={:.2}",
        params.height, params.weight, params.muscle, params.age
    );

    // Save session before running pipeline (captures params regardless of pipeline result)
    if let Some(ref session_path) = save_session {
        let mut session = MorphSession::new(&params);
        if let Some(ref td) = targets {
            session = session.with_targets_dir(td);
        }
        session
            .save(session_path)
            .with_context(|| format!("saving session to {}", session_path.display()))?;
        eprintln!("OxiHuman: session saved -> {}", session_path.display());
    }

    // Resolve expression preset if provided
    let expression_preset = if let Some(ref expr_name) = expression_name {
        let preset = ExpressionPreset::from_name(expr_name).ok_or_else(|| {
            anyhow::anyhow!(
                "unknown expression '{}'. Available: {}",
                expr_name,
                ExpressionPreset::all_names().join(", ")
            )
        })?;
        Some(preset)
    } else {
        None
    };

    let mesh = if let Some(ref expr_preset) = expression_preset {
        // Build with expression: manual engine pipeline
        build_mesh_with_expression(
            &base,
            targets.as_deref(),
            params,
            policy,
            &output,
            expr_preset,
        )?
    } else {
        // Standard pipeline
        let config = PipelineConfig {
            base_obj_path: base,
            targets_dir: targets,
            max_targets,
            policy,
            params,
            output_path: output.clone(),
        };
        run_pipeline(config)?
    };

    eprintln!(
        "OxiHuman: {} verts, {} faces -> {}",
        mesh.vertex_count(),
        mesh.face_count(),
        output.display()
    );

    if let Some(ref obj_path) = output_obj {
        export_obj(&mesh, obj_path)?;
        eprintln!("OxiHuman: OBJ -> {}", obj_path.display());
    }

    if measurements {
        let meas = export_mesh_measurements(&mesh);
        println!("{}", serde_json::to_string_pretty(&meas)?);
    }

    if let Some(ref qpath) = quantize_output {
        let qmesh = quantize_mesh(&mesh);
        let bytes = write_quantized_bin(&qmesh, qpath)
            .with_context(|| format!("writing QMSH to {}", qpath.display()))?;
        println!(
            "Written QMSH: {} vertices, {} indices → {} bytes",
            qmesh.positions.len(),
            qmesh.indices.len(),
            bytes
        );
    }

    Ok(())
}
