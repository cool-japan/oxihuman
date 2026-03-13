pub mod galvanic_export;
pub use galvanic_export::{
    gsr_detect_responses, gsr_mean_conductance, gsr_peak_conductance, gsr_to_bytes, gsr_to_csv,
    new_gsr_sample, GsrSample,
};

pub mod strain_field_export;
pub use strain_field_export::{
    new_strain_field, strain_exceeds_threshold, strain_get, strain_max, strain_mean, strain_set,
    strain_to_bytes, StrainField,
};

pub mod medical_dicom_export;
pub use medical_dicom_export::{
    dicom_get_pixel, dicom_hu_to_display, dicom_pixel_count, dicom_set_pixel, dicom_to_bytes,
    new_dicom_slice, DicomSlice,
};

pub mod brain_signal_export;
pub use brain_signal_export::{
    eeg_band_power, eeg_duration_s, eeg_push_sample, eeg_rms, eeg_to_bytes, eeg_to_csv,
    new_eeg_channel, EegChannel,
};

pub mod muscle_activation_export;
pub use muscle_activation_export::{
    activation_duration_s, activation_mean, activation_peak, activation_push, activation_to_bytes,
    activation_to_csv, new_muscle_activation, MuscleActivation,
};

pub mod joint_torque_export;
pub use joint_torque_export::{
    new_joint_torque, torque_duration_s, torque_mean_magnitude, torque_peak, torque_push,
    torque_to_csv, JointTorque,
};

pub mod ground_reaction_export;
pub use ground_reaction_export::{
    grf_duration_s, grf_impulse, grf_peak_vertical, grf_push, grf_to_csv, new_ground_reaction,
    GroundReactionForce,
};

pub mod center_of_mass_export;
pub use center_of_mass_export::{
    com_duration_s, com_mean_height, com_push, com_to_csv, com_total_distance, new_center_of_mass,
    CenterOfMass,
};

pub mod inertia_tensor_export;
pub use inertia_tensor_export::{
    inertia_is_symmetric, inertia_principal_moments, inertia_set_diagonal, inertia_to_bytes,
    inertia_to_json, new_inertia_tensor, InertiaTensor,
};

pub mod skin_deformation_export;
pub use skin_deformation_export::{
    new_skin_deform_map, skin_deform_get, skin_deform_max_stretch, skin_deform_rms,
    skin_deform_set, skin_deform_to_bytes, SkinDeformMap,
};

pub mod ao_map_export;
pub use ao_map_export::{
    ao_get, ao_mean, ao_set, ao_to_bytes_req as ao_map_to_bytes, ao_to_u8, new_ao_map_req, AoMap,
};

pub mod roughness_map_export;
pub use roughness_map_export::{
    new_roughness_metalness_map, rm_get, rm_mean_roughness, rm_set, rm_to_bytes,
    RoughnessMetalnessMap,
};

pub mod displacement_map_export;
pub use displacement_map_export::{
    disp_get, disp_max_height, disp_set, disp_to_bytes, disp_to_u16, new_displacement_map,
    DisplacementMap,
};

pub mod opacity_map_export;
pub use opacity_map_export::{
    new_opacity_map, opacity_get, opacity_mean, opacity_set, opacity_threshold_mask, opacity_to_u8,
    OpacityMap,
};

pub mod emission_map_export;
pub use emission_map_export::{
    emission_get, emission_mean, emission_set, emission_to_bytes, emission_total_power,
    new_emission_map, EmissionMap,
};

pub mod subsurface_map_export;
pub use subsurface_map_export::{
    new_subsurface_map, sss_get_color, sss_get_radius, sss_set, sss_to_bytes, SubsurfaceMap,
};

pub mod wrinkle_map_export_data;
pub use wrinkle_map_export_data::{
    new_wrinkle_map_data, wrinkle_active_count, wrinkle_get, wrinkle_max_weight, wrinkle_set,
    wrinkle_to_bytes, WrinkleMapData,
};

pub mod ior_map_export;
pub use ior_map_export::{
    ior_get, ior_is_valid, ior_mean, ior_set, ior_to_bytes, new_ior_map, IorMap,
};

pub mod transmission_map_export;
pub use transmission_map_export::{
    new_transmission_map, trans_get, trans_mean, trans_set, trans_threshold_mask, trans_to_bytes,
    TransmissionMap,
};

pub mod scatter_coefficient_export;
pub use scatter_coefficient_export::{
    new_scatter_coefficients, scatter_albedo, scatter_count, scatter_extinction, scatter_push,
    scatter_to_csv, ScatterCoefficients,
};

pub mod skin_color_export;
pub use skin_color_export::{
    new_skin_color_spectrum, spectrum_count, spectrum_mean_reflectance, spectrum_push,
    spectrum_to_csv, spectrum_to_rgb, SkinColorSpectrum,
};

pub mod melanin_map_export;
pub use melanin_map_export::{
    melanin_map_get, melanin_map_mean_eu, melanin_map_set, melanin_map_to_bytes, melanin_map_total,
    new_melanin_map, MelaninMap,
};

pub mod hemoglobin_map_export;
pub use hemoglobin_map_export::{
    hemo_map_get, hemo_map_oxygen_saturation, hemo_map_set, hemo_map_to_bytes, new_hemoglobin_map,
    HemoglobinMap,
};

pub mod sebum_map_export;
pub use sebum_map_export::{
    new_sebum_map, sebum_get, sebum_mean, sebum_set, sebum_to_bytes, sebum_zones, SebumMap,
};

pub mod pore_map_export;
pub use pore_map_export::{
    new_pore_map, pore_count, pore_density, pore_get, pore_mean_size, pore_set, PoreMap,
};

pub mod skin_hair_root_export;
pub use skin_hair_root_export::{
    new_skin_hair_root, skin_hair_root_count, skin_hair_root_density_per_cm2,
    skin_hair_root_mean_diameter, skin_hair_root_to_csv_line, skin_hair_roots_to_csv, SkinHairRoot,
};

pub mod eyelash_export;
pub use eyelash_export::{
    eyelash_count, eyelash_length, eyelash_mean_length, eyelash_to_csv_line, eyelashes_to_csv,
    new_eyelash, Eyelash,
};

pub mod eyebrow_export;
pub use eyebrow_export::{
    eyebrow_density, eyebrow_hair_to_csv_line, eyebrow_hairs_to_csv, eyebrow_mean_length,
    new_eyebrow_hair, EyebrowHair,
};

pub mod beard_export;
pub use beard_export::{
    beard_count, beard_coverage_density, beard_mean_length, beard_strand_to_csv_line,
    beard_strands_to_csv, new_beard_strand, BeardStrand,
};

pub mod tattoo_map_export;
pub use tattoo_map_export::{
    new_tattoo_map, tattoo_coverage, tattoo_get_pixel, tattoo_set_pixel, tattoo_to_bytes, TattooMap,
};

pub mod scar_map_export;
pub use scar_map_export::{
    new_scar_map, scar_coverage, scar_get, scar_mean_elevation, scar_set, scar_to_bytes, ScarMap,
};

pub mod vein_map_export;
pub use vein_map_export::{
    new_vein_map, vein_add_path, vein_mean_depth, vein_path_count, vein_to_json, vein_total_length,
    VeinMap,
};

pub mod age_spot_export;
pub use age_spot_export::{
    new_age_spot, spot_area_mm2, spot_to_csv_line, spots_count, spots_mean_darkness, spots_to_csv,
    AgeSpot,
};

pub mod tooth_export;
pub use tooth_export::{
    new_tooth, teeth_to_json, tooth_count, tooth_is_molar, tooth_to_json, tooth_total_length, Tooth,
};

pub mod nail_export;
pub use nail_export::{
    nail_area_mm2, nail_count, nail_is_long, nail_to_json, nails_to_json, new_nail, Nail,
};

pub mod microstructure_export;
pub use microstructure_export::{
    micro_age_index, micro_is_smooth, micro_roughness_index, micro_skin_type_estimate,
    micro_to_json, new_skin_microstructure, SkinMicrostructure,
};

pub mod nla_strip_export;
pub use nla_strip_export::{
    new_nla_strip, strip_duration, strip_overlaps, strip_to_json, strips_to_json, NlaStrip,
};

pub mod light_export;
pub use light_export::{
    default_point_light_export, light_is_directional, light_lux_at_distance, light_to_json,
    lights_to_json, new_light_data, LightData, LightExport, LIGHT_DIRECTIONAL, LIGHT_POINT,
    LIGHT_SPOT,
};

pub mod camera_export;
pub use camera_export::{
    camera_is_orthographic, camera_projection_matrix, camera_to_json, camera_view_distance,
    default_camera_export, new_camera_data, CameraData, CameraExport,
};

pub mod world_export;
pub use world_export::{
    new_world_data, world_ambient_energy, world_fog_visibility, world_has_hdri, world_to_json,
    WorldData,
};

pub mod freestyle_export;
pub use freestyle_export::{
    freestyle_push_point, freestyle_stroke_count, freestyle_stroke_length,
    freestyle_strokes_to_svg, new_freestyle_stroke, FreestyleStroke,
};

pub mod cryptomatte_export;
pub use cryptomatte_export::{
    cryptomatte_coverage_sum, cryptomatte_entries_to_json, cryptomatte_name_to_hash,
    cryptomatte_to_json, new_cryptomatte_entry, CryptomatteEntry,
};

pub mod deep_image_export;
pub use deep_image_export::{
    deep_image_sample_count, deep_pixel_flatten, deep_pixel_push, new_deep_image, new_deep_pixel,
    DeepImage, DeepPixel,
};

pub mod constraint_export;
pub use constraint_export::{
    constraint_count, constraint_is_active, constraint_is_active_spec as constraint_is_active_data,
    constraint_kind_name, constraint_set_influence, constraint_to_json, constraint_validate,
    constraints_to_json, new_constraint_data, new_constraint_export, ConstraintData,
    ConstraintExport, ConstraintKind,
};

pub mod driver_export;
pub use driver_export::{
    driver_add_coefficient, driver_coefficient_count, driver_evaluate, driver_has_expression,
    driver_push_variable, driver_to_json as driver_data_to_json, driver_type_name, driver_validate,
    driver_variable_count, new_driver_data, new_driver_export, DriverData, DriverExport,
    DriverType,
};

pub mod render_pass_export;
pub use render_pass_export::{
    add_render_pass, new_render_pass, new_render_pass_export, pass_get_pixel, pass_mean,
    pass_set_pixel, pass_to_bytes, RenderPass, RenderPassEntry, RenderPassExport,
};

pub mod grease_pencil_export;
pub use grease_pencil_export::{
    gp_point_count, gp_push_point, gp_stroke_length, gp_stroke_to_json, gp_strokes_to_json,
    new_gp_stroke, new_grease_pencil_export, GpLayer, GpStroke, GreasePencilExport,
};

pub mod collection_export;
pub use collection_export::{
    collection_child_count, collection_object_count, collection_push_child, collection_push_object,
    collection_to_json, new_collection_export, new_collection_node, CollectionExport,
    CollectionNode, CollectionObject,
};

pub mod compositor_export;
pub use compositor_export::{
    comp_node_is_output, comp_node_to_json, comp_nodes_to_json, comp_push_input, comp_push_output,
    default_compositor_export, export_compositor_to_json, new_compositor_node, CompositorExport,
    CompositorNode,
};

// --- Wave 151B additions ---
pub mod gltf2_export;
pub use gltf2_export::{
    asset_to_json, gltf2_scene_json, new_gltf2_asset, new_gltf2_node, node_to_json, nodes_to_json,
    Gltf2Asset, Gltf2Node,
};

pub mod vrm_export;
pub use vrm_export::{
    VrmBoneName, VrmCommercialUsage as VrmCommercialUsage10, VrmCreditNotation, VrmExporter,
    VrmHumanBone, VrmHumanoid as VrmHumanoid10, VrmMeta as VrmMeta10, VrmModification,
};

pub mod openxr_export;
pub use openxr_export::{
    new_xr_skeleton, xr_is_hand_skeleton, xr_joint_count, xr_push_joint, xr_to_json, XrSkeleton,
};

pub mod mixamo_export;
pub use mixamo_export::{
    mixamo_bone_to_json, mixamo_is_standard_bone, mixamo_rig_to_json, mixamo_standard_bones,
    new_mixamo_bone, MixamoBone,
};

pub mod smpl_export;
pub use smpl_export::{
    new_smpl_params, smpl_gender_name, smpl_param_count, smpl_set_beta, smpl_set_pose,
    smpl_to_json, SmplParams,
};

pub mod mediapipe_export;
pub use mediapipe_export::{
    new_mediapipe_landmark, new_mediapipe_pose, pose_is_complete, pose_landmark_name,
    pose_push_landmark, pose_to_json, MediapipeLandmark, MediapipePose,
};

pub mod openpose_export;
pub use openpose_export::{
    body_is_valid_coco, body_push_keypoint, body_to_json, keypoint_name_coco, new_openpose_body,
    new_openpose_keypoint, OpenPoseBody, OpenPoseKeypoint,
};

pub mod daz3d_export;
pub use daz3d_export::{
    daz_is_genesis8, daz_morph_count, daz_push_morph, daz_to_json, new_daz_figure, DazFigureExport,
};

pub mod makehuman_export;
pub use makehuman_export::{
    mh_find_param, mh_param_count, mh_push_param, mh_to_mhm_string, new_mh_export, MhExport,
    MhMorphParam,
};

pub mod cmumotion_export;
pub use cmumotion_export::{
    cmu_duration_s, cmu_frame_count, cmu_push_frame, cmu_to_csv, new_cmu_motion, CmuFrame,
    CmuMotion,
};

pub mod h36m_export;
pub use h36m_export::{
    h36m_joint_count, h36m_joint_name, h36m_push_joint, h36m_to_csv_line, new_h36m_skeleton,
    H36mSkeleton,
};

pub mod panoptic_export;
pub use panoptic_export::{
    new_panoptic_body, panoptic_is_body25, panoptic_keypoint_count, panoptic_push_keypoint,
    panoptic_to_json, PanopticBody,
};

pub mod smplx_export;
pub use smplx_export::{
    new_smplx_params, smplx_num_betas, smplx_num_expression, smplx_set_expression, smplx_to_json,
    SmplxParams,
};

pub mod flame_export;
pub use flame_export::{
    flame_expression_count, flame_set_shape, flame_shape_count, flame_to_json, new_flame_params,
    FlameParams,
};

pub mod tdmm_export;
pub use tdmm_export::{
    new_tdmm_params, tdmm_exp_count, tdmm_id_count, tdmm_reconstruct_stub, tdmm_to_json, TdmmParams,
};

pub mod dense_pose_export;
pub use dense_pose_export::{
    dense_pose_coverage, dense_pose_get, dense_pose_set, dense_pose_to_bytes,
    new_dense_pose_result, DensePoseResult,
};

pub mod three_js_export;
pub use three_js_export::{
    add_geometry, add_object, export_threejs_to_json, new_three_js_scene, ThreeJsGeometry,
    ThreeJsObject, ThreeJsScene,
};

pub mod babylon_export;
pub use babylon_export::{
    babylon_material_count, babylon_mesh_count, new_babylon_export, to_babylon_json, BabylonExport,
    BabylonMaterial, BabylonMesh,
};

pub mod a_frame_export;
pub use a_frame_export::{
    aframe_add_box, aframe_add_sphere, aframe_entity_count, aframe_push_entity,
    export_mesh_as_aframe, new_aframe_scene, render_aframe_html, validate_aframe, AFrameEntity,
    AFrameScene,
};

pub mod openscad_export;
pub use openscad_export::{
    export_mesh_as_openscad, new_openscad_export, openscad_node_count, openscad_push_node,
    render_openscad, validate_openscad, OpenScadExport, ScadNode, ScadPrim,
};

pub mod scad_export;
pub use scad_export::{
    new_scad_export, render_scad, scad_add_box, scad_add_cylinder, scad_add_metadata,
    scad_add_sphere, scad_bounding_box, scad_prim_count, validate_scad, ScadExport, ScadPrimType,
    ScadPrimitive,
};

pub mod dotobj_export;
pub use dotobj_export::{
    dotobj_add_material, dotobj_face_count, dotobj_material_count, dotobj_set_mesh,
    new_dotobj_export, render_dotmtl, render_dotobj, validate_dotobj, DotObjExport, DotObjMaterial,
};

pub mod ply_binary_export;
pub use ply_binary_export::{
    export_ply_binary, new_ply_binary_export, ply_binary_face_bytes, ply_binary_face_count,
    ply_binary_header, ply_binary_set_mesh, ply_binary_size_estimate, ply_binary_vertex_bytes,
    ply_binary_vertex_count, validate_ply_binary, PlyBinaryExport,
};

pub mod wrl_export;
pub use wrl_export::{
    export_mesh_as_wrl, new_wrl_document, render_wrl, validate_wrl, wrl_add_shape, wrl_shape_count,
    wrl_size_estimate, wrl_total_vertex_count, WrlDocument, WrlShape,
};

pub mod x_export;
pub use x_export::{
    new_x_document, render_x_document, validate_x_document, x_add_mesh, x_file_header,
    x_mesh_count, x_mesh_from_geometry, x_size_estimate, x_total_face_count, x_total_vertex_count,
    XDocument, XMesh,
};

pub mod ac3d_export;
pub use ac3d_export::{
    ac3d_add_material, ac3d_add_mesh, ac3d_material_count, ac3d_object_count, ac3d_size_estimate,
    new_ac3d_export, render_ac3d, validate_ac3d, Ac3dExport, Ac3dMaterial, Ac3dObject, Ac3dSurface,
};

pub mod lwo_export;
pub use lwo_export::{
    lwo_add_layer, lwo_add_surface, lwo_form_header, lwo_layer_count, lwo_size_estimate,
    lwo_surface_count, lwo_total_polygon_count, lwo_total_vertex_count, new_lwo_export,
    render_lwo_summary, validate_lwo, LwoExport, LwoLayer, LwoSurface,
};

pub mod nff_export;
pub use nff_export::{
    new_nff_document, nff_add_light, nff_add_mesh, nff_add_polygon, nff_add_sphere,
    nff_light_count, nff_polygon_count, nff_primitive_count, nff_set_background, nff_size_estimate,
    nff_sphere_count, render_nff, validate_nff, NffDocument, NffLight, NffPolygon, NffSphere,
    NffSurface,
};

pub mod markdown_report_export;
pub use markdown_report_export::{
    add_md_row as report_add_md_row, add_md_section, default_body_report as default_body_md_report,
    export_markdown_report, markdown_byte_count, md_row_count as report_md_row_count,
    md_section_count, new_markdown_report, render_markdown, validate_markdown_report,
    MarkdownReport, MarkdownSection,
};

pub mod html_report_export;
pub use html_report_export::{
    add_measurement as html_add_measurement, add_note as html_add_note, default_html_body_report,
    export_html_body_report, html_report_size_bytes, measurement_count as html_measurement_count,
    new_html_body_report, note_count as html_note_count, render_html_report, validate_html_report,
    HtmlBodyReport,
};

pub mod dot_graph_export;
pub use dot_graph_export::{
    dot_size_bytes, export_dot, new_dot_graph, render_dot, scene_to_dot, validate_dot_graph,
    DotGraph,
};

pub mod svg_skeleton_export;
pub use svg_skeleton_export::{
    add_svg_bone, bone_length_px, default_biped_svg_skeleton, export_svg_skeleton,
    new_svg_skeleton_doc, render_svg_skeleton, svg_bone_count, svg_skeleton_size_bytes,
    validate_svg_skeleton_doc, SvgBone, SvgSkeletonDoc,
};

pub mod pdf_metadata_export;
pub use pdf_metadata_export::{
    default_pdf_metadata_for_body_report, new_pdf_metadata, pdf_add_keyword, pdf_keyword_count,
    pdf_metadata_to_info_dict, pdf_metadata_to_xmp, pdf_set_page_count, pdf_set_subject,
    validate_pdf_metadata, PdfMetadata,
};

pub mod toml_config_export;
pub use toml_config_export::{
    default_export_config_toml, export_toml, new_toml_table, render_toml_table, toml_add_sub_table,
    toml_entry_count, toml_set_bool, toml_set_float, toml_set_int, toml_set_string,
    toml_size_bytes, validate_toml_table, TomlTable, TomlValue,
};

pub mod ron_export;
pub use ron_export::{
    export_ron, render_ron, ron_bool, ron_float, ron_int, ron_list, ron_list_len, ron_map,
    ron_map_get, ron_none, ron_size_bytes, ron_some, ron_str, ron_struct, scene_to_ron,
    validate_ron_value, RonValue,
};

pub mod binary_stl_export;
pub use binary_stl_export::{
    add_binary_stl_triangle, binary_stl_size_bytes, binary_stl_triangle_count, encode_binary_stl,
    mesh_to_binary_stl, new_binary_stl_mesh, parse_binary_stl_header, validate_binary_stl,
    BinaryStlMesh, BinaryStlTriangle,
};

pub mod ascii_stl_export;
pub use ascii_stl_export::{
    ascii_stl_size_bytes, count_ascii_stl_triangles, default_ascii_stl_options, export_ascii_stl,
    parse_ascii_stl_vertices, render_ascii_stl, validate_ascii_stl, AsciiStlOptions,
};

pub mod opensim_export;
pub use opensim_export::{
    add_opensim_body, add_opensim_joint, add_opensim_muscle, default_biped_opensim_model,
    export_opensim, new_opensim_model, opensim_body_count, opensim_muscle_count,
    opensim_size_bytes, render_opensim_xml, validate_opensim_model, OpenSimBody, OpenSimJoint,
    OpenSimModel, OpenSimMuscle,
};

pub mod opensim_ik_export;
pub use opensim_ik_export::{
    add_ik_coordinate_task, add_ik_marker_task, default_ik_setup, export_opensim_ik,
    ik_coordinate_task_count, ik_marker_task_count, ik_set_time_range, new_opensim_ik_setup,
    opensim_ik_size_bytes, render_opensim_ik_xml, validate_opensim_ik, IkCoordinateTask, IkMarker,
    OpenSimIkSetup,
};
