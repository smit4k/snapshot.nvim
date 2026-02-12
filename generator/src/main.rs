use ab_glyph::{FontVec, PxScale};
use anyhow::{Context, Result};
use arboard::Clipboard;
use chrono::offset::Local;
use chrono::DateTime;
use image::{ImageBuffer, Rgba, RgbaImage};
use imageproc::drawing::draw_text_mut;
use serde::Deserialize;
use std::io::{self, Read};

#[derive(Debug, Deserialize)]
struct Span {
    start: usize,
    end: usize,
    fg: Option<String>,
    bg: Option<String>,
    bold: Option<bool>,
    italic: Option<bool>,
    underline: Option<bool>,
    undercurl: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct Line {
    text: String,
    spans: Vec<Span>,
}

#[derive(Debug, Deserialize)]
struct Config {
    #[serde(default)]
    snapshot_dir: Option<String>,
    #[serde(default)]
    output_path: Option<String>,
    #[serde(default = "default_scale")]
    scale: f32,
    #[serde(default = "default_padding")]
    padding: u32,
    #[serde(default = "default_line_height")]
    line_height: f32,
    #[serde(default = "default_font_size")]
    font_size: f32,
    #[serde(default = "default_background")]
    background: String,
    #[serde(default = "default_foreground")]
    foreground: String,
    #[serde(default = "default_clipboard")]
    clipboard: bool,
    #[serde(default = "default_shadow")]
    shadow: bool,
    #[serde(default = "default_line_numbers")]
    line_numbers: bool,
    #[serde(default = "default_start_line")]
    start_line: usize,
    #[serde(default = "default_border_radius")]
    border_radius: u32,
}

fn default_padding() -> u32 {
    80
}
fn default_scale() -> f32 {
    2.0
}
fn default_line_height() -> f32 {
    28.0
}
fn default_font_size() -> f32 {
    20.0
}
fn default_background() -> String {
    "#282c34".to_string()
}
fn default_foreground() -> String {
    "#abb2bf".to_string()
}
fn default_clipboard() -> bool {
    true
}
fn default_shadow() -> bool {
    true
}
fn default_line_numbers() -> bool {
    false
}
fn default_start_line() -> usize {
    1
}
fn default_border_radius() -> u32 {
    5
}

#[derive(Debug, Deserialize)]
struct Input {
    lines: Vec<Line>,
    config: Config,
}

fn hex_to_rgba(hex: &str) -> Rgba<u8> {
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

fn measure_text_width(text: &str, font: &FontVec, scale: PxScale) -> u32 {
    use ab_glyph::{Font, ScaleFont};

    let scaled_font = font.as_scaled(scale);
    let mut width = 0.0;

    for c in text.chars() {
        let glyph_id = scaled_font.glyph_id(c);
        let advance = scaled_font.h_advance(glyph_id);
        width += advance;
    }

    width.ceil() as u32
}

fn apply_rounded_corners(img: &mut RgbaImage, radius: u32) {
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

/// Single-pass horizontal box blur on an alpha-only buffer stored row-major.
fn box_blur_h(src: &[u8], dst: &mut [u8], w: usize, h: usize, radius: usize) {
    let diam = radius * 2 + 1;
    for y in 0..h {
        let row = y * w;
        // running sum seeded with the left edge repeated into the margin
        let mut sum: u32 = (radius as u32 + 1) * src[row] as u32;
        for i in 0..radius {
            sum += src[row + i.min(w - 1)] as u32;
        }
        for x in 0..w {
            let right = (x + radius).min(w - 1);
            let left = (x as isize - radius as isize - 1).max(0) as usize;
            sum += src[row + right] as u32;
            dst[row + x] = (sum / diam as u32).min(255) as u8;
            sum -= src[row + left] as u32;
        }
    }
}

/// Single-pass vertical box blur on an alpha-only buffer stored row-major.
fn box_blur_v(src: &[u8], dst: &mut [u8], w: usize, h: usize, radius: usize) {
    let diam = radius * 2 + 1;
    for x in 0..w {
        let mut sum: u32 = (radius as u32 + 1) * src[x] as u32;
        for i in 0..radius {
            sum += src[i.min(h - 1) * w + x] as u32;
        }
        for y in 0..h {
            let bottom = (y + radius).min(h - 1);
            let top = (y as isize - radius as isize - 1).max(0) as usize;
            sum += src[bottom * w + x] as u32;
            dst[y * w + x] = (sum / diam as u32).min(255) as u8;
            sum -= src[top * w + x] as u32;
        }
    }
}

/// Approximate Gaussian blur via three successive box-blur passes on an alpha buffer.
fn gaussian_blur_alpha(buf: &mut Vec<u8>, w: usize, h: usize, sigma: f32) {
    // Compute box radius for 3-pass approximation of Gaussian
    // See: "Fast Almost-Gaussian Filtering" – W. Wells (1986)
    let boxes = boxes_for_gauss(sigma, 3);
    let mut tmp = vec![0u8; w * h];

    for &box_size in &boxes {
        let radius = ((box_size - 1) / 2) as usize;
        if radius == 0 {
            continue;
        }
        box_blur_h(buf, &mut tmp, w, h, radius);
        box_blur_v(&tmp, buf, w, h, radius);
    }
}

/// Compute ideal box sizes for n-pass box blur approximating a Gaussian with given sigma.
fn boxes_for_gauss(sigma: f32, n: usize) -> Vec<i32> {
    let w_ideal = ((12.0 * sigma * sigma / n as f32) + 1.0).sqrt();
    let mut wl = w_ideal.floor() as i32;
    if wl % 2 == 0 {
        wl -= 1;
    }
    let wu = wl + 2;

    let m_ideal = (12.0 * sigma * sigma
        - (n as f32 * wl as f32 * wl as f32)
        - 4.0 * n as f32 * wl as f32
        - 3.0 * n as f32)
        / (-4.0 * wl as f32 - 4.0);
    let m = m_ideal.round() as usize;

    (0..n).map(|i| if i < m { wl } else { wu }).collect()
}

/// Compose a drop-shadow behind `card` and return the final image.
///
/// The shadow is created by:
/// 1. Extracting the card's alpha channel as a silhouette
/// 2. Blurring the silhouette with a Gaussian approximation
/// 3. Tinting the blurred silhouette black at `shadow_opacity`
/// 4. Placing the blurred shadow on a transparent canvas, offset slightly downward
/// 5. Compositing the original card on top
fn apply_outer_shadow(
    card: &RgbaImage,
    shadow_blur: f32,
    shadow_opacity: f32,
    offset_x: i32,
    offset_y: i32,
) -> RgbaImage {
    let (cw, ch) = card.dimensions();
    let margin = (shadow_blur * 3.0).ceil() as u32; // extra space around card for the blur

    let out_w = cw + margin * 2;
    let out_h = ch + margin * 2;

    // Build an alpha-only buffer from the card's alpha channel, placed at margin offset
    let buf_w = out_w as usize;
    let buf_h = out_h as usize;
    let mut alpha_buf = vec![0u8; buf_w * buf_h];

    for cy in 0..ch {
        for cx in 0..cw {
            // Place shadow silhouette shifted by (offset_x, offset_y) relative to center
            let dx = (margin as i32 + offset_x + cx as i32) as usize;
            let dy = (margin as i32 + offset_y + cy as i32) as usize;
            if dx < buf_w && dy < buf_h {
                let a = card.get_pixel(cx, cy)[3];
                alpha_buf[dy * buf_w + dx] = a;
            }
        }
    }

    // Blur the alpha buffer
    gaussian_blur_alpha(&mut alpha_buf, buf_w, buf_h, shadow_blur);

    // Compose: start with transparent canvas, paint blurred shadow, then overlay card
    let mut output: RgbaImage = ImageBuffer::from_pixel(out_w, out_h, Rgba([0, 0, 0, 0]));

    // Paint blurred shadow (black, modulated by blurred alpha * opacity)
    for y in 0..out_h {
        for x in 0..out_w {
            let a = alpha_buf[y as usize * buf_w + x as usize] as f32 / 255.0;
            let shadow_a = (a * shadow_opacity * 255.0) as u8;
            if shadow_a > 0 {
                output.put_pixel(x, y, Rgba([0, 0, 0, shadow_a]));
            }
        }
    }

    // Composite the card on top at (margin, margin)
    for cy in 0..ch {
        for cx in 0..cw {
            let src = card.get_pixel(cx, cy);
            let sa = src[3] as f32 / 255.0;
            if sa <= 0.0 {
                continue;
            }
            let dx = cx + margin;
            let dy = cy + margin;
            let dst = output.get_pixel(dx, dy);
            let da = dst[3] as f32 / 255.0;

            // Standard Porter-Duff "source over" compositing
            let out_a = sa + da * (1.0 - sa);
            if out_a <= 0.0 {
                continue;
            }
            let r = (src[0] as f32 * sa + dst[0] as f32 * da * (1.0 - sa)) / out_a;
            let g = (src[1] as f32 * sa + dst[1] as f32 * da * (1.0 - sa)) / out_a;
            let b = (src[2] as f32 * sa + dst[2] as f32 * da * (1.0 - sa)) / out_a;
            output.put_pixel(
                dx,
                dy,
                Rgba([r as u8, g as u8, b as u8, (out_a * 255.0) as u8]),
            );
        }
    }

    output
}

fn generate_image(input: Input) -> Result<()> {
    let config = input.config;
    let lines = input.lines;

    // Determine output path
    let output_path = if let Some(path) = config.output_path {
        // User provided explicit output_path
        path
    } else {
        // Generate timestamped filename
        let now: DateTime<Local> = Local::now();
        let formatted_time = now.format("%Y-%m-%d_%H-%M-%S").to_string();
        let filename = format!("snapshot-{}.png", formatted_time);

        // Use snapshot_dir if provided, otherwise use $HOME
        if let Some(dir) = config.snapshot_dir {
            format!("{}/{}", dir, filename)
        } else {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            format!("{}/{}", home, filename)
        }
    };

    // Font setup - use a monospace font
    let font_data = include_bytes!("../fonts/JetBrainsMono-Regular.ttf");
    let font = FontVec::try_from_vec(font_data.to_vec()).context("Failed to load font")?;

    // Apply resolution scale factor for crisp/HiDPI rendering.
    // All rendering dimensions are multiplied by `scale` so text is rasterized
    // at a higher resolution, producing sharp output on Retina displays.
    let render_scale = if config.scale > 0.0 {
        config.scale
    } else {
        2.0
    };
    let scaled_font_size = config.font_size * render_scale;
    let scaled_padding = (config.padding as f32 * render_scale) as u32;
    let scaled_line_height = config.line_height * render_scale;

    let scale = PxScale::from(scaled_font_size);

    // Calculate image dimensions (all in scaled pixels)
    let line_number_width = if config.line_numbers {
        let max_line_num = config.start_line + lines.len();
        let line_num = format!("{:>4}  ", max_line_num);
        measure_text_width(&line_num, &font, scale)
    } else {
        0
    };

    // Calculate actual content width based on longest line
    let max_content_width = lines
        .iter()
        .map(|line| measure_text_width(&line.text, &font, scale))
        .max()
        .unwrap_or((800.0 * render_scale) as u32);

    let width = max_content_width + line_number_width + scaled_padding * 2;
    let height = (lines.len() as f32 * scaled_line_height) as u32 + scaled_padding * 2;

    // Create image at scaled resolution
    let bg_color = hex_to_rgba(&config.background);
    let mut img: RgbaImage = ImageBuffer::from_pixel(width, height, bg_color);

    // Default text color from the editor's Normal highlight group foreground
    let default_fg = hex_to_rgba(&config.foreground);

    // Draw lines with syntax highlighting (all coordinates in scaled pixels)
    for (line_idx, line) in lines.iter().enumerate() {
        // Calculate Y position with proper vertical alignment
        let y = scaled_padding as i32 + (line_idx as f32 * scaled_line_height) as i32;
        let mut x = scaled_padding as f32;

        // Draw line numbers
        if config.line_numbers {
            let line_num = format!("{:>4}  ", config.start_line + line_idx);
            let line_num_color = hex_to_rgba("#5c6370");
            draw_text_mut(
                &mut img,
                line_num_color,
                x as i32,
                y,
                scale,
                &font,
                &line_num,
            );
            x += line_number_width as f32;
        }

        // If no spans, just draw the whole line in default color
        if line.spans.is_empty() {
            draw_text_mut(&mut img, default_fg, x as i32, y, scale, &font, &line.text);
            continue;
        }

        // Draw spans with colors
        let mut last_end = 0;
        for span in &line.spans {
            // Draw unstyled text before this span
            if span.start > last_end {
                let unstyled_text = &line.text[last_end..span.start];
                draw_text_mut(
                    &mut img,
                    default_fg,
                    x as i32,
                    y,
                    scale,
                    &font,
                    unstyled_text,
                );
                let text_width = measure_text_width(unstyled_text, &font, scale);
                x += text_width as f32;
            }

            // Draw the span
            let span_text = &line.text[span.start..span.end.min(line.text.len())];
            let color = span
                .fg
                .as_ref()
                .map(|c| hex_to_rgba(c))
                .unwrap_or(default_fg);

            draw_text_mut(&mut img, color, x as i32, y, scale, &font, span_text);

            let text_width = measure_text_width(span_text, &font, scale);
            x += text_width as f32;
            last_end = span.end;
        }

        // Draw any remaining unstyled text
        if last_end < line.text.len() {
            let remaining_text = &line.text[last_end..];
            draw_text_mut(
                &mut img,
                default_fg,
                x as i32,
                y,
                scale,
                &font,
                remaining_text,
            );
        }
    }

    // Apply rounded corners by masking the alpha channel
    if config.border_radius > 0 {
        let scaled_radius = (config.border_radius as f32 * render_scale) as u32;
        apply_rounded_corners(&mut img, scaled_radius);
    }

    // Apply outer drop-shadow when enabled
    let img = if config.shadow {
        let shadow_sigma = 20.0 * render_scale; // blur radius scaled for HiDPI
        let shadow_opacity = 0.5;
        let offset_x = 0;
        let offset_y = (8.0 * render_scale) as i32; // slight downward offset
        apply_outer_shadow(&img, shadow_sigma, shadow_opacity, offset_x, offset_y)
    } else {
        img
    };

    // Expand tilde and environment variables in output path
    let expanded_path = shellexpand::full(&output_path)
        .context("Failed to expand output path")?
        .to_string();

    // Create parent directories if they don't exist
    if let Some(parent) = std::path::Path::new(&expanded_path).parent() {
        std::fs::create_dir_all(parent).context("Failed to create parent directories")?;
    }

    if config.clipboard {
        let img_data = arboard::ImageData {
            width: img.width() as usize,
            height: img.height() as usize,
            bytes: img.as_raw().into(),
        };

        let mut clipboard = Clipboard::new().unwrap();

        clipboard.set_image(img_data).unwrap_or_else(|e| {
            eprintln!("Warning: Failed to copy to clipboard: {}", e);
        });
    }

    // Save image
    img.save(&expanded_path).context("Failed to save image")?;

    println!("{}", expanded_path);
    Ok(())
}

fn main() -> Result<()> {
    // Read JSON from stdin
    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .context("Failed to read from stdin")?;

    let input: Input = serde_json::from_str(&buffer).context("Failed to parse JSON input")?;

    generate_image(input)?;
    Ok(())
}
