use ab_glyph::{Font, ScaleFont};
use image::{Rgba, RgbaImage};

const HEX_FALLBACK: Rgba<u8> = Rgba([255, 255, 255, 255]);

fn parse_hex_component(component: &str) -> Option<u8> {
    if component.len() != 2 {
        return None;
    }

    u8::from_str_radix(component, 16).ok()
}

pub fn try_hex_to_rgba(hex: &str) -> Option<Rgba<u8>> {
    let hex = hex.trim().trim_start_matches('#');

    match hex.len() {
        6 => Some(Rgba([
            parse_hex_component(&hex[0..2])?,
            parse_hex_component(&hex[2..4])?,
            parse_hex_component(&hex[4..6])?,
            255,
        ])),
        8 => Some(Rgba([
            parse_hex_component(&hex[0..2])?,
            parse_hex_component(&hex[2..4])?,
            parse_hex_component(&hex[4..6])?,
            parse_hex_component(&hex[6..8])?,
        ])),
        _ => None,
    }
}

pub fn hex_to_rgba(hex: &str) -> Rgba<u8> {
    try_hex_to_rgba(hex).unwrap_or(HEX_FALLBACK)
}

pub fn measure_text_width<F, SF>(text: &str, font: &SF) -> u32
where
    F: Font,
    SF: ScaleFont<F>,
{
    let width = text.chars().fold(0.0, |acc, c| {
        let glyph_id = font.glyph_id(c);
        acc + font.h_advance(glyph_id)
    });

    width.ceil() as u32
}

pub fn apply_rounded_corners(img: &mut RgbaImage, radius: u32) {
    let (width, height) = img.dimensions();
    let radius = radius.min(width / 2).min(height / 2);
    if radius == 0 {
        return;
    }

    let right_start = width - radius;
    let bottom_start = height - radius;
    let top_left = (radius as f32 - 0.5, radius as f32 - 0.5);
    let top_right = (width as f32 - radius as f32 - 0.5, radius as f32 - 0.5);
    let bottom_left = (radius as f32 - 0.5, height as f32 - radius as f32 - 0.5);
    let bottom_right = (
        width as f32 - radius as f32 - 0.5,
        height as f32 - radius as f32 - 0.5,
    );

    apply_corner(img, 0, 0, radius, top_left);
    apply_corner(img, right_start, 0, radius, top_right);
    apply_corner(img, 0, bottom_start, radius, bottom_left);
    apply_corner(img, right_start, bottom_start, radius, bottom_right);
}

fn apply_corner(img: &mut RgbaImage, start_x: u32, start_y: u32, radius: u32, center: (f32, f32)) {
    let radius_f = radius as f32;

    for y in start_y..start_y + radius {
        for x in start_x..start_x + radius {
            let dx = x as f32 + 0.5 - center.0;
            let dy = y as f32 + 0.5 - center.1;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance <= radius_f - 1.5 {
                continue;
            }

            let pixel = img.get_pixel_mut(x, y);
            if distance > radius_f {
                pixel[3] = 0;
            } else {
                let alpha = ((radius_f - distance) / 1.5).clamp(0.0, 1.0);
                pixel[3] = (pixel[3] as f32 * alpha) as u8;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{apply_rounded_corners, hex_to_rgba, try_hex_to_rgba};
    use image::{ImageBuffer, Rgba};

    #[test]
    fn parses_rgb_and_rgba_hex_values() {
        assert_eq!(
            try_hex_to_rgba("#112233"),
            Some(Rgba([0x11, 0x22, 0x33, 0xff]))
        );
        assert_eq!(
            try_hex_to_rgba("11223344"),
            Some(Rgba([0x11, 0x22, 0x33, 0x44]))
        );
    }

    #[test]
    fn invalid_hex_falls_back_to_white() {
        assert_eq!(try_hex_to_rgba("#12"), None);
        assert_eq!(try_hex_to_rgba("#zzzzzz"), None);
        assert_eq!(hex_to_rgba("#12"), Rgba([255, 255, 255, 255]));
    }

    #[test]
    fn rounded_corners_only_change_corner_pixels() {
        let mut image = ImageBuffer::from_pixel(8, 8, Rgba([1, 2, 3, 255]));
        apply_rounded_corners(&mut image, 3);

        assert!(image.get_pixel(0, 0)[3] < 255);
        assert_eq!(image.get_pixel(4, 4)[3], 255);
        assert!(image.get_pixel(2, 0)[3] < 255);
    }
}
