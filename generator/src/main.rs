mod shadow;
mod utils;

use ab_glyph::{Font, FontVec, PxScale};
use anyhow::{anyhow, Context, Result};
use arboard::Clipboard;
use chrono::offset::Local;
use chrono::DateTime;
use image::{ImageBuffer, Rgba, RgbaImage};
use imageproc::drawing::draw_text_mut;
use serde::Deserialize;
use std::io::{self, Read};
use std::path::PathBuf;

use shadow::{apply_outer_shadow, composite_image_onto};
use utils::{apply_rounded_corners, hex_to_rgba, measure_text_width};

const FALLBACK_RENDER_SCALE: f32 = 2.0;
const LINE_NUMBER_COLOR_HEX: &str = "#5c6370";

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
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
    #[serde(default = "default_outer_padding")]
    outer_padding: u32,
}

fn default_padding() -> u32 {
    25
}
fn default_scale() -> f32 {
    2.0
}
fn default_line_height() -> f32 {
    28.0
}
fn default_font_size() -> f32 {
    24.0
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
fn default_outer_padding() -> u32 {
    15
}

#[derive(Debug, Deserialize)]
struct Input {
    lines: Vec<Line>,
    config: Config,
}

struct RenderConfig {
    render_scale: f32,
    scale: PxScale,
    scaled_padding: u32,
    scaled_line_height: f32,
    scaled_outer_padding: u32,
    bg_color: Rgba<u8>,
    default_fg: Rgba<u8>,
    outer_bg: Rgba<u8>,
    line_number_color: Rgba<u8>,
}

struct PreparedSegment {
    start: usize,
    end: usize,
    width: u32,
    color: Rgba<u8>,
}

struct PreparedLine {
    width: u32,
    segments: Vec<PreparedSegment>,
}

fn render_scale(scale: f32) -> f32 {
    if scale > 0.0 {
        scale
    } else {
        FALLBACK_RENDER_SCALE
    }
}

fn resolve_output_path(config: &Config) -> String {
    if let Some(path) = &config.output_path {
        return path.clone();
    }

    let now: DateTime<Local> = Local::now();
    let formatted_time = now.format("%Y-%m-%d_%H-%M-%S").to_string();
    let filename = format!("snapshot-{formatted_time}.png");

    if let Some(dir) = &config.snapshot_dir {
        format!("{dir}/{filename}")
    } else {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        format!("{home}/{filename}")
    }
}

fn font_path() -> Result<PathBuf> {
    let current_exe = std::env::current_exe().context("Failed to locate generator binary")?;
    let parent = current_exe
        .parent()
        .ok_or_else(|| anyhow!("Generator binary has no parent directory"))?;
    Ok(parent.join("JetBrainsMono-Regular.ttf"))
}

fn load_font() -> Result<FontVec> {
    let path = font_path()?;
    let font_data = std::fs::read(&path).with_context(|| {
        format!(
            "Font not found at {}. Please reinstall the plugin to download the font.",
            path.display()
        )
    })?;
    FontVec::try_from_vec(font_data).context("Failed to load font")
}

fn build_render_config(config: &Config) -> RenderConfig {
    let render_scale = render_scale(config.scale);
    let scaled_font_size = config.font_size * render_scale;

    RenderConfig {
        render_scale,
        scale: PxScale::from(scaled_font_size),
        scaled_padding: (config.padding as f32 * render_scale) as u32,
        scaled_line_height: config.line_height * render_scale,
        scaled_outer_padding: (config.outer_padding as f32 * render_scale) as u32,
        bg_color: hex_to_rgba(&config.background),
        default_fg: hex_to_rgba(&config.foreground),
        outer_bg: hex_to_rgba(&config.outer_background),
        line_number_color: hex_to_rgba(LINE_NUMBER_COLOR_HEX),
    }
}

fn normalize_index(text: &str, index: usize) -> usize {
    let mut index = index.min(text.len());
    while index > 0 && !text.is_char_boundary(index) {
        index -= 1;
    }
    index
}

fn normalize_span_range(
    text: &str,
    start: usize,
    end: usize,
    last_end: usize,
) -> Option<(usize, usize)> {
    let start = normalize_index(text, start.max(last_end));
    let end = normalize_index(text, end);
    if start < end {
        Some((start, end))
    } else {
        None
    }
}

fn push_segment<F, SF>(
    segments: &mut Vec<PreparedSegment>,
    text: &str,
    start: usize,
    end: usize,
    color: Rgba<u8>,
    scaled_font: &SF,
) where
    F: Font,
    SF: ab_glyph::ScaleFont<F>,
{
    if start >= end {
        return;
    }

    let width = measure_text_width(&text[start..end], scaled_font);
    segments.push(PreparedSegment {
        start,
        end,
        width,
        color,
    });
}

fn prepare_line<F, SF>(line: &Line, scaled_font: &SF, default_fg: Rgba<u8>) -> PreparedLine
where
    F: Font,
    SF: ab_glyph::ScaleFont<F>,
{
    let mut segments = Vec::with_capacity(line.spans.len().saturating_mul(2).max(1));
    let mut last_end = 0;

    for span in &line.spans {
        let Some((start, end)) = normalize_span_range(&line.text, span.start, span.end, last_end)
        else {
            continue;
        };

        if last_end < start {
            push_segment(
                &mut segments,
                &line.text,
                last_end,
                start,
                default_fg,
                scaled_font,
            );
        }

        let color = span.fg.as_deref().map(hex_to_rgba).unwrap_or(default_fg);
        push_segment(&mut segments, &line.text, start, end, color, scaled_font);
        last_end = end;
    }

    if last_end < line.text.len() {
        push_segment(
            &mut segments,
            &line.text,
            last_end,
            line.text.len(),
            default_fg,
            scaled_font,
        );
    }

    let width = segments.iter().map(|segment| segment.width).sum();
    PreparedLine { width, segments }
}

fn line_number_width<F, SF>(config: &Config, lines: &[Line], scaled_font: &SF) -> u32
where
    F: Font,
    SF: ab_glyph::ScaleFont<F>,
{
    if !config.line_numbers {
        return 0;
    }

    let max_line_num = config.start_line + lines.len();
    let line_num = format!("{max_line_num:>4}  ");
    measure_text_width(&line_num, scaled_font)
}

fn measure_layout<F, SF>(
    lines: &[Line],
    config: &Config,
    render: &RenderConfig,
    scaled_font: &SF,
) -> (Vec<PreparedLine>, u32, u32, u32)
where
    F: Font,
    SF: ab_glyph::ScaleFont<F>,
{
    let prepared_lines: Vec<_> = lines
        .iter()
        .map(|line| prepare_line(line, scaled_font, render.default_fg))
        .collect();

    let line_number_width = line_number_width(config, lines, scaled_font);
    let max_content_width = prepared_lines
        .iter()
        .map(|line| line.width)
        .max()
        .unwrap_or((800.0 * render.render_scale) as u32);

    let width = max_content_width + line_number_width + render.scaled_padding * 2;
    let height =
        (lines.len() as f32 * render.scaled_line_height) as u32 + render.scaled_padding * 2;

    (prepared_lines, line_number_width, width, height)
}

fn render_card(
    lines: &[Line],
    prepared_lines: &[PreparedLine],
    config: &Config,
    render: &RenderConfig,
    font: &FontVec,
    line_number_width: u32,
    width: u32,
    height: u32,
) -> RgbaImage {
    let mut image: RgbaImage = ImageBuffer::from_pixel(width, height, render.bg_color);

    for (line_idx, (line, prepared)) in lines.iter().zip(prepared_lines).enumerate() {
        let y = render.scaled_padding as i32 + (line_idx as f32 * render.scaled_line_height) as i32;
        let mut x = render.scaled_padding as f32;

        if config.line_numbers {
            let line_num = format!("{:>4}  ", config.start_line + line_idx);
            draw_text_mut(
                &mut image,
                render.line_number_color,
                x as i32,
                y,
                render.scale,
                font,
                &line_num,
            );
            x += line_number_width as f32;
        }

        for segment in &prepared.segments {
            let text = &line.text[segment.start..segment.end];
            draw_text_mut(
                &mut image,
                segment.color,
                x as i32,
                y,
                render.scale,
                font,
                text,
            );
            x += segment.width as f32;
        }
    }

    image
}

fn apply_card_effects(mut image: RgbaImage, config: &Config, render: &RenderConfig) -> RgbaImage {
    if config.border_radius > 0 {
        let scaled_radius = (config.border_radius as f32 * render.render_scale) as u32;
        apply_rounded_corners(&mut image, scaled_radius);
    }
    image
}

fn finalize_output(card: &RgbaImage, config: &Config, render: &RenderConfig) -> RgbaImage {
    let mut image = if config.shadow {
        let shadow_sigma = 20.0 * render.render_scale;
        let shadow_opacity = 0.5;
        let offset_x = 0;
        let offset_y = (8.0 * render.render_scale) as i32;
        apply_outer_shadow(
            card,
            shadow_sigma,
            shadow_opacity,
            offset_x,
            offset_y,
            render.outer_bg,
            render.scaled_outer_padding,
        )
    } else {
        let margin = render.scaled_outer_padding;
        let (cw, ch) = card.dimensions();
        let out_w = cw + margin * 2;
        let out_h = ch + margin * 2;
        let mut output: RgbaImage = ImageBuffer::from_pixel(out_w, out_h, render.outer_bg);
        composite_image_onto(&mut output, card, margin, margin);
        output
    };

    if config.border_radius > 0 {
        let scaled_radius = (config.border_radius as f32 * render.render_scale) as u32;
        apply_rounded_corners(&mut image, scaled_radius);
    }

    image
}

fn copy_to_clipboard(image: &RgbaImage) -> Result<()> {
    let img_data = arboard::ImageData {
        width: image.width() as usize,
        height: image.height() as usize,
        bytes: image.as_raw().into(),
    };

    let mut clipboard = Clipboard::new().context("Failed to access clipboard")?;
    clipboard
        .set_image(img_data)
        .map_err(|error| anyhow!("Failed to copy to clipboard: {error}"))
}

fn save_image(image: &RgbaImage, output_path: &str) -> Result<String> {
    let expanded_path = shellexpand::full(output_path)
        .context("Failed to expand output path")?
        .to_string();

    if let Some(parent) = std::path::Path::new(&expanded_path).parent() {
        std::fs::create_dir_all(parent).context("Failed to create parent directories")?;
    }

    image.save(&expanded_path).context("Failed to save image")?;
    Ok(expanded_path)
}

fn generate_image(input: Input) -> Result<()> {
    let output_path = resolve_output_path(&input.config);
    let font = load_font()?;
    let render = build_render_config(&input.config);
    let scaled_font = font.as_scaled(render.scale);

    let (prepared_lines, line_number_width, width, height) =
        measure_layout(&input.lines, &input.config, &render, &scaled_font);
    let card = render_card(
        &input.lines,
        &prepared_lines,
        &input.config,
        &render,
        &font,
        line_number_width,
        width,
        height,
    );
    let card = apply_card_effects(card, &input.config, &render);
    let image = finalize_output(&card, &input.config, &render);

    if input.config.clipboard {
        copy_to_clipboard(&image).unwrap_or_else(|error| {
            eprintln!("Warning: {error}");
        });
    }

    let saved_path = save_image(&image, &output_path)?;
    println!("{saved_path}");
    Ok(())
}

fn main() -> Result<()> {
    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .context("Failed to read from stdin")?;

    let input: Input = serde_json::from_str(&buffer).context("Failed to parse JSON input")?;
    generate_image(input)
}

#[cfg(test)]
mod tests {
    use super::{normalize_index, normalize_span_range, render_scale, resolve_output_path, Config};

    fn base_config() -> Config {
        Config {
            snapshot_dir: None,
            output_path: None,
            scale: 2.0,
            padding: 25,
            line_height: 28.0,
            font_size: 24.0,
            background: "#282c34".to_string(),
            foreground: "#abb2bf".to_string(),
            clipboard: false,
            shadow: true,
            line_numbers: false,
            start_line: 1,
            border_radius: 5,
            outer_background: "#ffffff".to_string(),
            outer_padding: 15,
        }
    }

    #[test]
    fn output_path_prefers_explicit_value() {
        let mut config = base_config();
        config.output_path = Some("/tmp/out.png".to_string());

        assert_eq!(resolve_output_path(&config), "/tmp/out.png");
    }

    #[test]
    fn invalid_scale_uses_fallback() {
        assert_eq!(render_scale(0.0), 2.0);
        assert_eq!(render_scale(-1.0), 2.0);
        assert_eq!(render_scale(1.5), 1.5);
    }

    #[test]
    fn normalize_index_moves_to_char_boundary() {
        let text = "aé";
        assert_eq!(normalize_index(text, 2), 1);
        assert_eq!(normalize_index(text, 3), 3);
    }

    #[test]
    fn normalize_span_range_skips_invalid_or_overlapping_ranges() {
        let text = "hello";
        assert_eq!(normalize_span_range(text, 1, 4, 0), Some((1, 4)));
        assert_eq!(normalize_span_range(text, 0, 1, 2), None);
        assert_eq!(normalize_span_range(text, 8, 10, 0), None);
    }
}
