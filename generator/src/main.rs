mod shadow;
mod utils;

use ab_glyph::{FontVec, PxScale};
use anyhow::{Context, Result};
use arboard::Clipboard;
use chrono::offset::Local;
use chrono::DateTime;
use image::{ImageBuffer, RgbaImage};
use imageproc::drawing::draw_text_mut;
use serde::Deserialize;
use std::io::{self, Read};

use shadow::apply_outer_shadow;
use utils::{apply_rounded_corners, hex_to_rgba, measure_text_width};

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
    #[serde(default = "default_outer_background")]
    outer_background: String,
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
fn default_outer_background() -> String {
    "#ffffff".to_string()
}

#[derive(Debug, Deserialize)]
struct Input {
    lines: Vec<Line>,
    config: Config,
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
    let outer_bg = hex_to_rgba(&config.outer_background);
    let img = if config.shadow {
        let shadow_sigma = 20.0 * render_scale; // blur radius scaled for HiDPI
        let shadow_opacity = 0.5;
        let offset_x = 0;
        let offset_y = (8.0 * render_scale) as i32; // slight downward offset
        apply_outer_shadow(
            &img,
            shadow_sigma,
            shadow_opacity,
            offset_x,
            offset_y,
            outer_bg,
        )
    } else {
        // No shadow: add a small margin with the outer background color
        let margin = (scaled_padding as f32 * 0.5) as u32;
        let (cw, ch) = img.dimensions();
        let out_w = cw + margin * 2;
        let out_h = ch + margin * 2;
        let mut output: RgbaImage = ImageBuffer::from_pixel(out_w, out_h, outer_bg);
        // Composite the card onto the outer background
        for cy in 0..ch {
            for cx in 0..cw {
                let src = img.get_pixel(cx, cy);
                let sa = src[3] as f32 / 255.0;
                if sa <= 0.0 {
                    continue;
                }
                let dx = cx + margin;
                let dy = cy + margin;
                let dst = output.get_pixel(dx, dy);
                let da = dst[3] as f32 / 255.0;
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
                    image::Rgba([r as u8, g as u8, b as u8, (out_a * 255.0) as u8]),
                );
            }
        }
        output
    };

    // Apply rounded corners to the final output image (outer background included)
    let mut img = img;
    if config.border_radius > 0 {
        let scaled_radius = (config.border_radius as f32 * render_scale) as u32;
        apply_rounded_corners(&mut img, scaled_radius);
    }

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
