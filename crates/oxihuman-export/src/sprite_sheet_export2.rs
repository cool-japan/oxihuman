//! Sprite sheet export.
#![allow(dead_code)]

/// A single sprite frame.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct SpriteFrame2 {
    pub name: String,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

/// A sprite sheet composed of frames.
#[allow(dead_code)]
pub struct SpriteSheet2 {
    pub frames: Vec<SpriteFrame2>,
    pub sheet_width: u32,
    pub sheet_height: u32,
}

/// Create a new sprite sheet.
#[allow(dead_code)]
pub fn new_sprite_sheet2(width: u32, height: u32) -> SpriteSheet2 {
    SpriteSheet2 { frames: Vec::new(), sheet_width: width, sheet_height: height }
}

/// Add a frame to the sprite sheet.
#[allow(dead_code)]
pub fn add_frame2(sheet: &mut SpriteSheet2, frame: SpriteFrame2) {
    sheet.frames.push(frame);
}

/// Export sprite sheet to JSON string.
#[allow(dead_code)]
pub fn export_sprite_sheet2(sheet: &SpriteSheet2) -> String {
    let frames: Vec<String> = sheet.frames.iter().map(|f| {
        format!(r#"{{"name":"{}","x":{},"y":{},"w":{},"h":{}}}"#, f.name, f.x, f.y, f.w, f.h)
    }).collect();
    format!(r#"{{"width":{},"height":{},"frames":[{}]}}"#, sheet.sheet_width, sheet.sheet_height, frames.join(","))
}

/// Get the number of sprites.
#[allow(dead_code)]
pub fn sprite2_count(sheet: &SpriteSheet2) -> usize { sheet.frames.len() }

/// Get frame at index.
#[allow(dead_code)]
pub fn frame2_at(sheet: &SpriteSheet2, i: usize) -> Option<&SpriteFrame2> { sheet.frames.get(i) }

/// Get the UV rect of a frame (normalized 0..1).
#[allow(dead_code)]
pub fn sprite2_uv_rect(sheet: &SpriteSheet2, i: usize) -> Option<[f32; 4]> {
    let f = sheet.frames.get(i)?;
    let sw = sheet.sheet_width as f32; let sh = sheet.sheet_height as f32;
    Some([f.x as f32 / sw, f.y as f32 / sh, f.w as f32 / sw, f.h as f32 / sh])
}

/// Get sheet width.
#[allow(dead_code)]
pub fn sprite_sheet2_width(sheet: &SpriteSheet2) -> u32 { sheet.sheet_width }

/// Get sheet height.
#[allow(dead_code)]
pub fn sprite_sheet2_height(sheet: &SpriteSheet2) -> u32 { sheet.sheet_height }

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_frame(name: &str) -> SpriteFrame2 {
        SpriteFrame2 { name: name.to_string(), x: 0, y: 0, w: 64, h: 64 }
    }

    #[test]
    fn test_new_sprite_sheet() {
        let s = new_sprite_sheet2(512, 512);
        assert_eq!(sprite2_count(&s), 0);
    }

    #[test]
    fn test_add_frame() {
        let mut s = new_sprite_sheet2(512, 512);
        add_frame2(&mut s, sample_frame("idle"));
        assert_eq!(sprite2_count(&s), 1);
    }

    #[test]
    fn test_frame_at() {
        let mut s = new_sprite_sheet2(512, 512);
        add_frame2(&mut s, sample_frame("walk"));
        let f = frame2_at(&s, 0).unwrap();
        assert_eq!(f.name, "walk");
    }

    #[test]
    fn test_frame_at_oob() {
        let s = new_sprite_sheet2(512, 512);
        assert!(frame2_at(&s, 0).is_none());
    }

    #[test]
    fn test_sprite_uv_rect() {
        let mut s = new_sprite_sheet2(512, 512);
        add_frame2(&mut s, SpriteFrame2 { name:"x".to_string(), x:0, y:0, w:256, h:256 });
        let uv = sprite2_uv_rect(&s, 0).unwrap();
        assert!((uv[2] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_sheet_width() {
        let s = new_sprite_sheet2(800, 600);
        assert_eq!(sprite_sheet2_width(&s), 800);
    }

    #[test]
    fn test_sheet_height() {
        let s = new_sprite_sheet2(800, 600);
        assert_eq!(sprite_sheet2_height(&s), 600);
    }

    #[test]
    fn test_export_sprite_sheet() {
        let s = new_sprite_sheet2(512, 512);
        let json = export_sprite_sheet2(&s);
        assert!(json.contains("512"));
    }

    #[test]
    fn test_sprite_frame_struct() {
        let f = SpriteFrame2 { name:"run".to_string(), x:64, y:64, w:32, h:32 };
        assert_eq!(f.w, 32);
    }
}
