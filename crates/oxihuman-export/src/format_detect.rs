//! Detect file format from magic bytes and extensions.

#[allow(dead_code)]
#[derive(Clone, PartialEq, Debug)]
pub enum DetectedFormat {
    Glb,
    Gltf,
    Obj,
    Fbx,
    Usdz,
    Ply,
    Stl,
    Collada,
    Png,
    Jpeg,
    Bmp,
    Tga,
    Hdr,
    Exr,
    Json,
    Xml,
    Csv,
    Binary,
    Unknown,
}

#[allow(dead_code)]
pub struct FormatInfo {
    pub format: DetectedFormat,
    pub confidence: f32,
    pub mime_type: String,
    pub extensions: Vec<String>,
}

#[allow(dead_code)]
pub fn glb_magic() -> [u8; 4] {
    [0x67, 0x6C, 0x54, 0x46]
}

#[allow(dead_code)]
pub fn png_magic() -> [u8; 8] {
    [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]
}

#[allow(dead_code)]
pub fn detect_from_bytes(data: &[u8]) -> DetectedFormat {
    if data.len() >= 4 {
        let magic4 = &data[..4];
        // GLB: "glTF"
        if magic4 == glb_magic() {
            return DetectedFormat::Glb;
        }
        // FBX: "Kayd"
        if magic4 == b"Kayd" {
            return DetectedFormat::Fbx;
        }
        // BMP: "BM"
        if &data[..2] == b"BM" {
            return DetectedFormat::Bmp;
        }
        // JPEG: FF D8 FF
        if data.len() >= 3 && data[0] == 0xFF && data[1] == 0xD8 && data[2] == 0xFF {
            return DetectedFormat::Jpeg;
        }
        // EXR: 0x76 0x2F 0x31 0x01
        if magic4 == [0x76, 0x2F, 0x31, 0x01] {
            return DetectedFormat::Exr;
        }
        // ZIP (USDZ is ZIP-based): PK\x03\x04
        if &data[..4] == b"PK\x03\x04" {
            return DetectedFormat::Usdz;
        }
    }
    if data.len() >= 8 {
        let magic8 = &data[..8];
        if magic8 == png_magic() {
            return DetectedFormat::Png;
        }
    }
    // Text-based detection
    if data.len() >= 6 {
        if data.starts_with(b"ply\n") || data.starts_with(b"ply\r") {
            return DetectedFormat::Ply;
        }
        if data.starts_with(b"solid ") || data.starts_with(b"solid\n") {
            return DetectedFormat::Stl;
        }
        if data.starts_with(b"<?xml") || data.starts_with(b"<COLL") {
            if data.windows(8).any(|w| w == b"COLLADA>") {
                return DetectedFormat::Collada;
            }
            return DetectedFormat::Xml;
        }
        if data.starts_with(b"{") || data.starts_with(b"[") {
            return DetectedFormat::Json;
        }
        if data.starts_with(b"v ") || data.starts_with(b"# ") || data.starts_with(b"mtllib") {
            return DetectedFormat::Obj;
        }
        if data.starts_with(b"#EXTM3U")
            || data.starts_with(b"#RADIANCE")
            || data.starts_with(b"#?RADIANCE")
        {
            return DetectedFormat::Hdr;
        }
    }
    if data.is_empty() {
        return DetectedFormat::Unknown;
    }
    // Check if binary (non-printable bytes in first 512)
    let sample = &data[..data.len().min(512)];
    let non_printable = sample
        .iter()
        .filter(|&&b| b < 0x09 || (b > 0x0D && b < 0x20))
        .count();
    if non_printable > sample.len() / 10 {
        DetectedFormat::Binary
    } else {
        DetectedFormat::Unknown
    }
}

#[allow(dead_code)]
pub fn detect_from_extension(ext: &str) -> DetectedFormat {
    extension_to_format(ext)
}

#[allow(dead_code)]
pub fn detect_from_path(path: &str) -> DetectedFormat {
    if let Some(dot_pos) = path.rfind('.') {
        let ext = &path[dot_pos..];
        let fmt = extension_to_format(ext);
        if fmt != DetectedFormat::Unknown {
            return fmt;
        }
    }
    DetectedFormat::Unknown
}

#[allow(dead_code)]
pub fn extension_to_format(ext: &str) -> DetectedFormat {
    match ext.to_lowercase().as_str() {
        ".glb" => DetectedFormat::Glb,
        ".gltf" => DetectedFormat::Gltf,
        ".obj" => DetectedFormat::Obj,
        ".fbx" => DetectedFormat::Fbx,
        ".usdz" => DetectedFormat::Usdz,
        ".ply" => DetectedFormat::Ply,
        ".stl" => DetectedFormat::Stl,
        ".dae" => DetectedFormat::Collada,
        ".png" => DetectedFormat::Png,
        ".jpg" | ".jpeg" => DetectedFormat::Jpeg,
        ".bmp" => DetectedFormat::Bmp,
        ".tga" => DetectedFormat::Tga,
        ".hdr" => DetectedFormat::Hdr,
        ".exr" => DetectedFormat::Exr,
        ".json" => DetectedFormat::Json,
        ".xml" => DetectedFormat::Xml,
        ".csv" => DetectedFormat::Csv,
        _ => DetectedFormat::Unknown,
    }
}

#[allow(dead_code)]
pub fn format_info(fmt: &DetectedFormat) -> FormatInfo {
    match fmt {
        DetectedFormat::Glb => FormatInfo {
            format: DetectedFormat::Glb,
            confidence: 1.0,
            mime_type: "model/gltf-binary".to_string(),
            extensions: vec![".glb".to_string()],
        },
        DetectedFormat::Gltf => FormatInfo {
            format: DetectedFormat::Gltf,
            confidence: 1.0,
            mime_type: "model/gltf+json".to_string(),
            extensions: vec![".gltf".to_string()],
        },
        DetectedFormat::Obj => FormatInfo {
            format: DetectedFormat::Obj,
            confidence: 0.9,
            mime_type: "model/obj".to_string(),
            extensions: vec![".obj".to_string()],
        },
        DetectedFormat::Fbx => FormatInfo {
            format: DetectedFormat::Fbx,
            confidence: 1.0,
            mime_type: "application/octet-stream".to_string(),
            extensions: vec![".fbx".to_string()],
        },
        DetectedFormat::Usdz => FormatInfo {
            format: DetectedFormat::Usdz,
            confidence: 0.9,
            mime_type: "model/vnd.usdz+zip".to_string(),
            extensions: vec![".usdz".to_string()],
        },
        DetectedFormat::Ply => FormatInfo {
            format: DetectedFormat::Ply,
            confidence: 1.0,
            mime_type: "application/octet-stream".to_string(),
            extensions: vec![".ply".to_string()],
        },
        DetectedFormat::Stl => FormatInfo {
            format: DetectedFormat::Stl,
            confidence: 0.9,
            mime_type: "model/stl".to_string(),
            extensions: vec![".stl".to_string()],
        },
        DetectedFormat::Collada => FormatInfo {
            format: DetectedFormat::Collada,
            confidence: 1.0,
            mime_type: "model/vnd.collada+xml".to_string(),
            extensions: vec![".dae".to_string()],
        },
        DetectedFormat::Png => FormatInfo {
            format: DetectedFormat::Png,
            confidence: 1.0,
            mime_type: "image/png".to_string(),
            extensions: vec![".png".to_string()],
        },
        DetectedFormat::Jpeg => FormatInfo {
            format: DetectedFormat::Jpeg,
            confidence: 1.0,
            mime_type: "image/jpeg".to_string(),
            extensions: vec![".jpg".to_string(), ".jpeg".to_string()],
        },
        DetectedFormat::Bmp => FormatInfo {
            format: DetectedFormat::Bmp,
            confidence: 1.0,
            mime_type: "image/bmp".to_string(),
            extensions: vec![".bmp".to_string()],
        },
        DetectedFormat::Tga => FormatInfo {
            format: DetectedFormat::Tga,
            confidence: 0.7,
            mime_type: "image/x-tga".to_string(),
            extensions: vec![".tga".to_string()],
        },
        DetectedFormat::Hdr => FormatInfo {
            format: DetectedFormat::Hdr,
            confidence: 0.9,
            mime_type: "image/vnd.radiance".to_string(),
            extensions: vec![".hdr".to_string()],
        },
        DetectedFormat::Exr => FormatInfo {
            format: DetectedFormat::Exr,
            confidence: 1.0,
            mime_type: "image/x-exr".to_string(),
            extensions: vec![".exr".to_string()],
        },
        DetectedFormat::Json => FormatInfo {
            format: DetectedFormat::Json,
            confidence: 0.8,
            mime_type: "application/json".to_string(),
            extensions: vec![".json".to_string()],
        },
        DetectedFormat::Xml => FormatInfo {
            format: DetectedFormat::Xml,
            confidence: 0.8,
            mime_type: "application/xml".to_string(),
            extensions: vec![".xml".to_string()],
        },
        DetectedFormat::Csv => FormatInfo {
            format: DetectedFormat::Csv,
            confidence: 0.7,
            mime_type: "text/csv".to_string(),
            extensions: vec![".csv".to_string()],
        },
        DetectedFormat::Binary => FormatInfo {
            format: DetectedFormat::Binary,
            confidence: 0.5,
            mime_type: "application/octet-stream".to_string(),
            extensions: vec![],
        },
        DetectedFormat::Unknown => FormatInfo {
            format: DetectedFormat::Unknown,
            confidence: 0.0,
            mime_type: "application/octet-stream".to_string(),
            extensions: vec![],
        },
    }
}

#[allow(dead_code)]
pub fn is_3d_format(fmt: &DetectedFormat) -> bool {
    matches!(
        fmt,
        DetectedFormat::Glb
            | DetectedFormat::Gltf
            | DetectedFormat::Obj
            | DetectedFormat::Fbx
            | DetectedFormat::Usdz
            | DetectedFormat::Ply
            | DetectedFormat::Stl
            | DetectedFormat::Collada
    )
}

#[allow(dead_code)]
pub fn is_image_format(fmt: &DetectedFormat) -> bool {
    matches!(
        fmt,
        DetectedFormat::Png
            | DetectedFormat::Jpeg
            | DetectedFormat::Bmp
            | DetectedFormat::Tga
            | DetectedFormat::Hdr
            | DetectedFormat::Exr
    )
}

#[allow(dead_code)]
pub fn is_text_format(fmt: &DetectedFormat) -> bool {
    matches!(
        fmt,
        DetectedFormat::Json
            | DetectedFormat::Xml
            | DetectedFormat::Csv
            | DetectedFormat::Gltf
            | DetectedFormat::Obj
            | DetectedFormat::Collada
    )
}

#[allow(dead_code)]
pub fn mime_type(fmt: &DetectedFormat) -> &'static str {
    match fmt {
        DetectedFormat::Glb => "model/gltf-binary",
        DetectedFormat::Gltf => "model/gltf+json",
        DetectedFormat::Obj => "model/obj",
        DetectedFormat::Fbx => "application/octet-stream",
        DetectedFormat::Usdz => "model/vnd.usdz+zip",
        DetectedFormat::Ply => "application/octet-stream",
        DetectedFormat::Stl => "model/stl",
        DetectedFormat::Collada => "model/vnd.collada+xml",
        DetectedFormat::Png => "image/png",
        DetectedFormat::Jpeg => "image/jpeg",
        DetectedFormat::Bmp => "image/bmp",
        DetectedFormat::Tga => "image/x-tga",
        DetectedFormat::Hdr => "image/vnd.radiance",
        DetectedFormat::Exr => "image/x-exr",
        DetectedFormat::Json => "application/json",
        DetectedFormat::Xml => "application/xml",
        DetectedFormat::Csv => "text/csv",
        DetectedFormat::Binary | DetectedFormat::Unknown => "application/octet-stream",
    }
}

#[allow(dead_code)]
pub fn format_name(fmt: &DetectedFormat) -> &'static str {
    match fmt {
        DetectedFormat::Glb => "GLB",
        DetectedFormat::Gltf => "glTF",
        DetectedFormat::Obj => "OBJ",
        DetectedFormat::Fbx => "FBX",
        DetectedFormat::Usdz => "USDZ",
        DetectedFormat::Ply => "PLY",
        DetectedFormat::Stl => "STL",
        DetectedFormat::Collada => "Collada",
        DetectedFormat::Png => "PNG",
        DetectedFormat::Jpeg => "JPEG",
        DetectedFormat::Bmp => "BMP",
        DetectedFormat::Tga => "TGA",
        DetectedFormat::Hdr => "HDR",
        DetectedFormat::Exr => "EXR",
        DetectedFormat::Json => "JSON",
        DetectedFormat::Xml => "XML",
        DetectedFormat::Csv => "CSV",
        DetectedFormat::Binary => "Binary",
        DetectedFormat::Unknown => "Unknown",
    }
}

#[allow(dead_code)]
pub fn all_3d_formats() -> Vec<DetectedFormat> {
    vec![
        DetectedFormat::Glb,
        DetectedFormat::Gltf,
        DetectedFormat::Obj,
        DetectedFormat::Fbx,
        DetectedFormat::Usdz,
        DetectedFormat::Ply,
        DetectedFormat::Stl,
        DetectedFormat::Collada,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_from_bytes_glb() {
        let magic = glb_magic();
        let data: Vec<u8> = magic
            .iter()
            .chain(b"\x00\x00\x00\x00".iter())
            .cloned()
            .collect();
        assert_eq!(detect_from_bytes(&data), DetectedFormat::Glb);
    }

    #[test]
    fn test_detect_from_bytes_png() {
        let magic = png_magic();
        let data: Vec<u8> = magic.iter().chain(b"\x00\x00".iter()).cloned().collect();
        assert_eq!(detect_from_bytes(&data), DetectedFormat::Png);
    }

    #[test]
    fn test_detect_from_bytes_jpeg() {
        let data: Vec<u8> = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00];
        assert_eq!(detect_from_bytes(&data), DetectedFormat::Jpeg);
    }

    #[test]
    fn test_detect_from_bytes_ply() {
        let data = b"ply\nformat ascii 1.0\n";
        assert_eq!(detect_from_bytes(data), DetectedFormat::Ply);
    }

    #[test]
    fn test_detect_from_bytes_json() {
        let data = b"{\"key\": \"value\"}";
        assert_eq!(detect_from_bytes(data), DetectedFormat::Json);
    }

    #[test]
    fn test_detect_from_extension_obj() {
        assert_eq!(detect_from_extension(".obj"), DetectedFormat::Obj);
    }

    #[test]
    fn test_detect_from_extension_glb() {
        assert_eq!(detect_from_extension(".glb"), DetectedFormat::Glb);
    }

    #[test]
    fn test_detect_from_extension_unknown() {
        assert_eq!(detect_from_extension(".xyz123"), DetectedFormat::Unknown);
    }

    #[test]
    fn test_detect_from_path_ply() {
        assert_eq!(detect_from_path("model.ply"), DetectedFormat::Ply);
    }

    #[test]
    fn test_detect_from_path_unknown() {
        assert_eq!(detect_from_path("file"), DetectedFormat::Unknown);
    }

    #[test]
    fn test_is_3d_format_glb() {
        assert!(is_3d_format(&DetectedFormat::Glb));
        assert!(is_3d_format(&DetectedFormat::Obj));
        assert!(!is_3d_format(&DetectedFormat::Png));
    }

    #[test]
    fn test_is_image_format_png() {
        assert!(is_image_format(&DetectedFormat::Png));
        assert!(is_image_format(&DetectedFormat::Jpeg));
        assert!(!is_image_format(&DetectedFormat::Glb));
    }

    #[test]
    fn test_mime_type() {
        assert_eq!(mime_type(&DetectedFormat::Png), "image/png");
        assert_eq!(mime_type(&DetectedFormat::Glb), "model/gltf-binary");
        assert_eq!(mime_type(&DetectedFormat::Json), "application/json");
    }

    #[test]
    fn test_format_name() {
        assert_eq!(format_name(&DetectedFormat::Glb), "GLB");
        assert_eq!(format_name(&DetectedFormat::Png), "PNG");
        assert_eq!(format_name(&DetectedFormat::Unknown), "Unknown");
    }

    #[test]
    fn test_all_3d_formats_non_empty() {
        let fmts = all_3d_formats();
        assert!(!fmts.is_empty());
        assert!(fmts.contains(&DetectedFormat::Glb));
        assert!(fmts.contains(&DetectedFormat::Ply));
    }

    #[test]
    fn test_glb_magic_length() {
        let magic = glb_magic();
        assert_eq!(magic.len(), 4);
        assert_eq!(&magic, b"glTF");
    }

    #[test]
    fn test_png_magic_length() {
        let magic = png_magic();
        assert_eq!(magic.len(), 8);
    }

    #[test]
    fn test_is_text_format() {
        assert!(is_text_format(&DetectedFormat::Json));
        assert!(is_text_format(&DetectedFormat::Xml));
        assert!(!is_text_format(&DetectedFormat::Glb));
    }
}
