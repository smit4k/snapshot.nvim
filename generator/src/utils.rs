use ab_glyph::{Font, FontVec, PxScale, ScaleFont};
use image::{Rgba, RgbaImage};

pub fn hex_to_rgba(hex: &str) -> Rgba<u8> {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);
    let a = if hex.len() > 6 {
        u8::from_str_radix(&hex[6..8], 16).unwrap_or(255)
    } else {
        255
    };
    Rgba([r, g, b, a])
}

pub fn measure_text_width(text: &str, font: &FontVec, scale: PxScale) -> u32 {
    let scaled_font = font.as_scaled(scale);
    let mut width = 0.0;

    for c in text.chars() {
        let glyph_id = scaled_font.glyph_id(c);
        let advance = scaled_font.h_advance(glyph_id);
        width += advance;
    }

    width.ceil() as u32
}

pub fn apply_rounded_corners(img: &mut RgbaImage, radius: u32) {
    let (width, height) = img.dimensions();
    let radius_f = radius as f32;

    // Iterate through each pixel in the image
    for y in 0..height {
        for x in 0..width {
            // Determine which corner (if any) this pixel belongs to
            let corner_info = if x < radius && y < radius {
                // Top-left corner: center at (radius-1, radius-1)
                Some((radius as f32 - 0.5, radius as f32 - 0.5))
            } else if x >= width - radius && y < radius {
                // Top-right corner
                Some((width as f32 - radius as f32 - 0.5, radius as f32 - 0.5))
            } else if x < radius && y >= height - radius {
                // Bottom-left corner
                Some((radius as f32 - 0.5, height as f32 - radius as f32 - 0.5))
            } else if x >= width - radius && y >= height - radius {
                // Bottom-right corner
                Some((
                    width as f32 - radius as f32 - 0.5,
                    height as f32 - radius as f32 - 0.5,
                ))
            } else {
                None
            };

            if let Some((corner_x, corner_y)) = corner_info {
                // Calculate distance from corner center to pixel center
                let dx = x as f32 + 0.5 - corner_x;
                let dy = y as f32 + 0.5 - corner_y;
                let distance = (dx * dx + dy * dy).sqrt();

                let pixel = img.get_pixel_mut(x, y);

                if distance > radius_f {
                    // Outside the rounded corner - make fully transparent
                    pixel[3] = 0;
                } else if distance > radius_f - 1.5 {
                    // Anti-aliasing on the edge for smooth corners
                    let alpha = ((radius_f - distance) / 1.5).clamp(0.0, 1.0);
                    pixel[3] = (pixel[3] as f32 * alpha) as u8;
                }
            }
        }
    }
}
