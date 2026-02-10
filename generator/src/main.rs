use ab_glyph::{FontVec, PxScale};
use anyhow::{Context, Result};
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
    #[serde(default = "default_output_path")]
    output_path: String,
    #[serde(default = "default_padding")]
    padding: u32,
    #[serde(default = "default_line_height")]
    line_height: f32,
    #[serde(default = "default_font_size")]
    font_size: f32,
    #[serde(default = "default_background")]
    background: String,
    #[serde(default = "default_shadow")]
    shadow: bool,
    #[serde(default = "default_line_numbers")]
    line_numbers: bool,
    #[serde(default = "default_start_line")]
    start_line: usize,
}

fn default_output_path() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    format!("{}/snapshot.png", home)
}
fn default_padding() -> u32 {
    80
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
fn default_shadow() -> bool {
    true
}
fn default_line_numbers() -> bool {
    false
}
fn default_start_line() -> usize {
    1
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

fn generate_image(input: Input) -> Result<()> {
    let config = input.config;
    let lines = input.lines;

    // Font setup - use a monospace font
    let font_data = include_bytes!("../fonts/FiraCode-Regular.ttf");
    let font = FontVec::try_from_vec(font_data.to_vec()).context("Failed to load font")?;

    let scale = PxScale::from(config.font_size);

    // Calculate image dimensions
    let max_line_length = lines.iter().map(|l| l.text.len()).max().unwrap_or(80);

    let line_number_width = if config.line_numbers {
        let max_line_num = config.start_line + lines.len();
        let line_num = format!("{:>4} ", max_line_num);
        measure_text_width(&line_num, &font, scale)
    } else {
        0
    };

    // Calculate actual content width based on longest line
    let max_content_width = lines
        .iter()
        .map(|line| measure_text_width(&line.text, &font, scale))
        .max()
        .unwrap_or(800);

    let width = max_content_width + line_number_width + config.padding * 2;
    let height = (lines.len() as f32 * config.line_height) as u32 + config.padding * 2;

    // Create image
    let bg_color = hex_to_rgba(&config.background);
    let mut img: RgbaImage = ImageBuffer::from_pixel(width, height, bg_color);

    // Draw shadow if enabled (disabled for now - can be added with a blur filter)
    // Shadow rendering is complex and optional, so we'll skip it for simplicity

    // Draw lines with syntax highlighting
    for (line_idx, line) in lines.iter().enumerate() {
        // Calculate Y position with proper vertical alignment
        let y = config.padding as i32 + (line_idx as f32 * config.line_height) as i32;
        let mut x = config.padding as f32;

        // Draw line numbers
        if config.line_numbers {
            let line_num = format!("{:>4} ", config.start_line + line_idx);
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
            let default_color = hex_to_rgba("#abb2bf");
            draw_text_mut(
                &mut img,
                default_color,
                x as i32,
                y,
                scale,
                &font,
                &line.text,
            );
            continue;
        }

        // Draw spans with colors
        let mut last_end = 0;
        for span in &line.spans {
            // Draw unstyled text before this span
            if span.start > last_end {
                let unstyled_text = &line.text[last_end..span.start];
                let default_color = hex_to_rgba("#abb2bf");
                draw_text_mut(
                    &mut img,
                    default_color,
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
                .unwrap_or(hex_to_rgba("#abb2bf"));

            draw_text_mut(&mut img, color, x as i32, y, scale, &font, span_text);

            let text_width = measure_text_width(span_text, &font, scale);
            x += text_width as f32;
            last_end = span.end;
        }

        // Draw any remaining unstyled text
        if last_end < line.text.len() {
            let remaining_text = &line.text[last_end..];
            let default_color = hex_to_rgba("#abb2bf");
            draw_text_mut(
                &mut img,
                default_color,
                x as i32,
                y,
                scale,
                &font,
                remaining_text,
            );
        }
    }

    // Save image
    img.save(&config.output_path)
        .context("Failed to save image")?;

    println!("{}", config.output_path);
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
