mod shadow;
mod utils;

use ab_glyph::{Font, FontVec, PxScale};
use anyhow::{anyhow, ensure, Context, Result};
use arboard::Clipboard;
use chrono::offset::Local;
use image::{ImageBuffer, ImageEncoder, ImageFormat, Pixel, Rgba, RgbaImage};
use imageproc::drawing::{draw_filled_rect_mut, draw_text_mut};
use imageproc::rect::Rect;
use serde::Deserialize;
use std::fs::{self, File};
use std::io::{self, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use shadow::{apply_outer_shadow, composite_image_onto, shadow_blur_margin};
use utils::{apply_rounded_corners, hex_to_rgba, measure_text_width};

const FALLBACK_RENDER_SCALE: f32 = 2.0;
const SHADOW_BLUR_SIGMA: f32 = 20.0;
const MAX_IMAGE_PIXELS: u64 = 40_000_000;
const MAX_SCALED_FONT_SIZE: f32 = 4096.0;
const LINE_NUMBER_COLOR_HEX: &str = "#5c6370";
const TRAFFIC_LIGHT_SPACING: f32 = 20.0;
const TRAFFIC_LIGHT_RADIUS: f32 = 6.0;
const TITLE_BAR_HEIGHT: f32 = 32.0;

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
    #[serde(default)]
    traffic_lights: bool,
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
    file_path: Option<String>,
}

struct RenderConfig {
    render_scale: f32,
    scale: PxScale,
    scaled_padding: u32,
    scaled_line_height: f32,
    scaled_outer_padding: u32,
    scaled_border_radius: u32,
    scaled_title_bar_height: u32,
    bg_color: Rgba<u8>,
    default_fg: Rgba<u8>,
    outer_bg: Rgba<u8>,
    line_number_color: Rgba<u8>,
}

struct PreparedSegment<'a> {
    text: &'a str,
    width: f64,
    color: Rgba<u8>,
}

struct PreparedLine<'a> {
    segments: Vec<PreparedSegment<'a>>,
}

impl PreparedLine<'_> {
    fn width(&self) -> f64 {
        self.segments.iter().map(|segment| segment.width).sum()
    }
}

struct PreparedLayout<'a> {
    lines: Vec<PreparedLine<'a>>,
    file_path_header: Option<FilePathHeader<'a>>,
    line_number_width: f64,
    line_number_digits: usize,
    header_height: u32,
    traffic_light_center_y: f32,
    width: u32,
    height: u32,
}

struct FilePathHeader<'a> {
    text: &'a str,
    scale: PxScale,
    x: u32,
}

fn render_scale(scale: f32) -> f32 {
    if scale > 0.0 {
        scale
    } else {
        FALLBACK_RENDER_SCALE
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

fn build_render_config(config: &Config) -> Result<RenderConfig> {
    ensure!(config.scale.is_finite(), "Scale must be finite");
    let render_scale = render_scale(config.scale);
    let scaled_font_size = config.font_size * render_scale;
    let scaled_line_height = config.line_height * render_scale;
    ensure!(
        scaled_font_size > 0.0 && scaled_font_size <= MAX_SCALED_FONT_SIZE,
        "Scaled font size must be positive and at most {MAX_SCALED_FONT_SIZE} pixels"
    );
    ensure!(
        scaled_line_height.is_finite() && scaled_line_height > 0.0,
        "Scaled line height must be positive and finite"
    );

    Ok(RenderConfig {
        render_scale,
        scale: PxScale::from(scaled_font_size),
        scaled_padding: (config.padding as f32 * render_scale) as u32,
        scaled_line_height,
        scaled_outer_padding: (config.outer_padding as f32 * render_scale) as u32,
        scaled_border_radius: (config.border_radius as f32 * render_scale) as u32,
        scaled_title_bar_height: if config.traffic_lights {
            (TITLE_BAR_HEIGHT * render_scale).ceil() as u32
        } else {
            0
        },
        bg_color: hex_to_rgba(&config.background),
        default_fg: hex_to_rgba(&config.foreground),
        outer_bg: hex_to_rgba(&config.outer_background),
        line_number_color: hex_to_rgba(LINE_NUMBER_COLOR_HEX),
    })
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

fn push_segment<'a, F, SF>(
    segments: &mut Vec<PreparedSegment<'a>>,
    text: &'a str,
    color: Rgba<u8>,
    scaled_font: &SF,
) where
    F: Font,
    SF: ab_glyph::ScaleFont<F>,
{
    let width = measure_text_width(text, scaled_font);
    segments.push(PreparedSegment { text, width, color });
}

fn prepare_line<'a, F, SF>(
    line: &'a Line,
    scaled_font: &SF,
    default_fg: Rgba<u8>,
) -> PreparedLine<'a>
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
                &line.text[last_end..start],
                default_fg,
                scaled_font,
            );
        }

        let color = span.fg.as_deref().map(hex_to_rgba).unwrap_or(default_fg);
        push_segment(&mut segments, &line.text[start..end], color, scaled_font);
        last_end = end;
    }

    if last_end < line.text.len() {
        push_segment(
            &mut segments,
            &line.text[last_end..],
            default_fg,
            scaled_font,
        );
    }

    PreparedLine { segments }
}

fn measure_layout<'a>(
    input: &'a Input,
    render: &RenderConfig,
    font: &FontVec,
) -> Result<PreparedLayout<'a>> {
    let lines = &input.lines;
    let config = &input.config;
    let scaled_font = font.as_scaled(render.scale);
    let prepared_lines: Vec<_> = lines
        .iter()
        .map(|line| prepare_line(line, &scaled_font, render.default_fg))
        .collect();

    let (line_number_digits, line_number_width) = if config.line_numbers {
        let last = config
            .start_line
            .checked_add(lines.len().saturating_sub(1))
            .context("Starting line number is too large for the capture")?;
        let digits = last.to_string().len().max(4);
        let label = format!("{last:>digits$}  ");
        (digits, measure_text_width(&label, &scaled_font))
    } else {
        (0, 0.0)
    };
    let max_content_width = prepared_lines
        .iter()
        .map(PreparedLine::width)
        .max_by(f64::total_cmp)
        .unwrap_or_else(|| 800.0 * f64::from(render.render_scale));

    let file_path = input.file_path.as_deref().filter(|path| !path.is_empty());
    let file_path_scale = PxScale::from(render.scale.y * 0.8);
    let (header_height, traffic_light_center_y) = if file_path.is_some() {
        let vertical_padding = (8.0 * render.render_scale).ceil();
        let path_height = (file_path_scale.y.ceil() + 2.0 * vertical_padding) as u32;
        let height = render.scaled_title_bar_height.max(path_height);
        (height, height as f32 / 2.0)
    } else {
        (
            render.scaled_title_bar_height,
            TITLE_BAR_HEIGHT * render.render_scale / 2.0,
        )
    };
    let padding = u64::from(render.scaled_padding) * 2;
    let height = u64::from((lines.len() as f32 * render.scaled_line_height) as u32)
        + padding
        + u64::from(header_height);
    let mut width = u64::from((max_content_width + line_number_width).ceil() as u32) + padding;
    if config.traffic_lights {
        width = width.max(u64::from(
            (TRAFFIC_LIGHT_SPACING * 4.0 * render.render_scale).ceil() as u32,
        ));
    }
    let file_path_header = file_path.map(|text| {
        let mut x = (f64::from(render.scaled_padding) + line_number_width).round();
        if config.traffic_lights {
            // Use the largest corner radius the card height allows while the width is still unknown.
            let corner_radius = u64::from(render.scaled_border_radius).min(height / 2) as f32;
            let first_x =
                first_traffic_light_x(render.render_scale, corner_radius, traffic_light_center_y);
            let after_lights = first_x
                + (2.0 * TRAFFIC_LIGHT_SPACING + TRAFFIC_LIGHT_RADIUS + 12.0) * render.render_scale;
            x = x.max(f64::from(after_lights.ceil()));
        }
        let path_width = measure_text_width(text, &font.as_scaled(file_path_scale)).max(1.0);
        let path_right = (x + path_width).ceil() as u32;
        width = width.max(u64::from(path_right) + u64::from(render.scaled_padding));
        FilePathHeader {
            text,
            scale: file_path_scale,
            x: x as u32,
        }
    });
    ensure!(
        width > 0 && height > 0,
        "Snapshot width and height must be positive"
    );

    let blur_margin = if config.shadow {
        shadow_blur_margin(SHADOW_BLUR_SIGMA * render.render_scale)
    } else {
        0
    };
    let outer_padding = 2 * (u64::from(blur_margin) + u64::from(render.scaled_outer_padding));
    let output_width = width + outer_padding;
    let output_height = height + outer_padding;
    ensure!(
        output_width
            .checked_mul(output_height)
            .is_some_and(|pixels| pixels <= MAX_IMAGE_PIXELS),
        "Snapshot dimensions {output_width} x {output_height} exceed the {MAX_IMAGE_PIXELS}-pixel limit; lower the scale or capture a smaller range"
    );

    Ok(PreparedLayout {
        lines: prepared_lines,
        file_path_header,
        line_number_width,
        line_number_digits,
        header_height,
        traffic_light_center_y,
        width: width as u32,
        height: height as u32,
    })
}

fn render_card(
    layout: &PreparedLayout<'_>,
    config: &Config,
    render: &RenderConfig,
    font: &FontVec,
) -> RgbaImage {
    let mut image = ImageBuffer::from_pixel(layout.width, layout.height, render.bg_color);
    let code_x = f64::from(render.scaled_padding) + layout.line_number_width;
    if let Some(header) = &layout.file_path_header {
        if !config.traffic_lights {
            let divider_x = code_x.round() as u32;
            let mut tint = render.default_fg;
            tint[3] /= 3;
            let mut divider = render.bg_color;
            divider.blend(&tint);
            let divider_height = (render.render_scale.round() as u32).max(1);
            let divider_width = layout.width - divider_x - render.scaled_padding;
            draw_filled_rect_mut(
                &mut image,
                Rect::at(
                    divider_x as i32,
                    (layout.header_height - divider_height) as i32,
                )
                .of_size(divider_width, divider_height),
                divider,
            );
        }

        draw_text_mut(
            &mut image,
            render.default_fg,
            header.x as i32,
            ((layout.header_height as f32 - header.scale.y) / 2.0).round() as i32,
            header.scale,
            font,
            header.text,
        );
    }

    let code_top = layout.header_height + render.scaled_padding;
    for (line_idx, line) in layout.lines.iter().enumerate() {
        let y = code_top as i32 + (line_idx as f32 * render.scaled_line_height) as i32;
        if config.line_numbers {
            let number = config.start_line + line_idx;
            let digits = layout.line_number_digits;
            let line_num = format!("{number:>digits$}  ");
            draw_text_mut(
                &mut image,
                render.line_number_color,
                render.scaled_padding as i32,
                y,
                render.scale,
                font,
                &line_num,
            );
        }

        let mut x = code_x;
        for segment in &line.segments {
            draw_text_mut(
                &mut image,
                segment.color,
                x.round() as i32,
                y,
                render.scale,
                font,
                segment.text,
            );
            x += segment.width;
        }
    }

    image
}

fn first_traffic_light_x(scale: f32, corner_radius: f32, center_y: f32) -> f32 {
    // Fit the first dot inside the corner arc when the card has a large radius.
    let inner_radius = (corner_radius - TRAFFIC_LIGHT_RADIUS * scale - 1.0).max(0.0);
    let corner_dy = (corner_radius - center_y).max(0.0);
    let corner_dx = (inner_radius.powi(2) - corner_dy.powi(2)).max(0.0).sqrt();
    (TRAFFIC_LIGHT_SPACING * scale).max(corner_radius - corner_dx)
}

fn draw_traffic_lights(image: &mut RgbaImage, scale: f32, border_radius: u32, center_y: f32) {
    let spacing = TRAFFIC_LIGHT_SPACING * scale;
    let radius = TRAFFIC_LIGHT_RADIUS * scale;
    let corner_radius = border_radius.min(image.width() / 2).min(image.height() / 2) as f32;
    let first_center_x = first_traffic_light_x(scale, corner_radius, center_y);

    let top = (center_y - radius).floor() as u32;
    let bottom = (center_y + radius).ceil() as u32;
    for (index, hex) in ["#ff5f57", "#febc2e", "#28c840"].into_iter().enumerate() {
        let center_x = first_center_x + index as f32 * spacing;
        let left = (center_x - radius).floor() as u32;
        let right = (center_x + radius).ceil() as u32;
        let mut color = hex_to_rgba(hex);

        for y in top..bottom.min(image.height()) {
            for x in left..right.min(image.width()) {
                let distance = (x as f32 + 0.5 - center_x).hypot(y as f32 + 0.5 - center_y);
                let coverage = (radius + 0.5 - distance).clamp(0.0, 1.0);
                color[3] = (coverage * 255.0).round() as u8;
                image.get_pixel_mut(x, y).blend(&color);
            }
        }
    }
}

fn apply_card_effects(
    mut image: RgbaImage,
    config: &Config,
    render: &RenderConfig,
    traffic_light_center_y: f32,
) -> RgbaImage {
    if config.traffic_lights {
        draw_traffic_lights(
            &mut image,
            render.render_scale,
            render.scaled_border_radius,
            traffic_light_center_y,
        );
    }
    apply_rounded_corners(&mut image, render.scaled_border_radius);
    image
}

fn finalize_output(card: &RgbaImage, config: &Config, render: &RenderConfig) -> RgbaImage {
    let mut image = if config.shadow {
        let shadow_sigma = SHADOW_BLUR_SIGMA * render.render_scale;
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

fn expand_path(path: &str) -> Result<PathBuf> {
    Ok(PathBuf::from(
        shellexpand::full(path)
            .with_context(|| format!("Failed to expand output path {path:?}"))?
            .as_ref(),
    ))
}

fn save_generated_png(image: &RgbaImage, directory: &Path, timestamp: &str) -> Result<PathBuf> {
    fs::create_dir_all(directory)
        .with_context(|| format!("Failed to create output directory {}", directory.display()))?;

    let mut suffix = 0;
    loop {
        let name = if suffix == 0 {
            format!("snapshot-{timestamp}.png")
        } else {
            format!("snapshot-{timestamp}-{suffix}.png")
        };
        let path = directory.join(name);
        let file = match File::create_new(&path) {
            Ok(file) => file,
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {
                suffix += 1;
                continue;
            }
            Err(err) => {
                return Err(err).with_context(|| format!("Failed to create {}", path.display()));
            }
        };

        let mut writer = BufWriter::new(file);
        let result = image::codecs::png::PngEncoder::new(&mut writer)
            .write_image(
                image.as_raw(),
                image.width(),
                image.height(),
                image::ExtendedColorType::Rgba8,
            )
            .map_err(anyhow::Error::from)
            .and_then(|()| writer.flush().map_err(anyhow::Error::from));
        drop(writer);

        if let Err(err) = result {
            let _ = fs::remove_file(&path);
            return Err(err).with_context(|| format!("Failed to save image to {}", path.display()));
        }
        return Ok(path);
    }
}

fn save_image(image: &RgbaImage, config: &Config) -> Result<PathBuf> {
    let directory = if let Some(output_path) = &config.output_path {
        let path = expand_path(output_path)?;
        if path.is_dir() || output_path.ends_with(std::path::is_separator) {
            path
        } else {
            let format = ImageFormat::from_path(&path)
                .with_context(|| format!("Unsupported image path {}", path.display()))?;
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).with_context(|| {
                    format!("Failed to create output directory {}", parent.display())
                })?;
            }

            let saved = if format == ImageFormat::Jpeg {
                let flattened = ImageBuffer::from_fn(image.width(), image.height(), |x, y| {
                    let mut pixel = Rgba([255; 4]);
                    pixel.blend(image.get_pixel(x, y));
                    pixel.to_rgb()
                });
                flattened.save_with_format(&path, format)
            } else {
                image.save_with_format(&path, format)
            };
            saved.with_context(|| format!("Failed to save image to {}", path.display()))?;
            return Ok(path);
        }
    } else if let Some(directory) = &config.snapshot_dir {
        expand_path(directory)?
    } else {
        std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .map_or_else(|| PathBuf::from("."), PathBuf::from)
    };

    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    save_generated_png(image, &directory, &timestamp)
}

fn generate_image(input: Input) -> Result<()> {
    let render = build_render_config(&input.config)?;
    let font = load_font()?;

    let layout = measure_layout(&input, &render, &font)?;
    let card = render_card(&layout, &input.config, &render, &font);
    let card = apply_card_effects(card, &input.config, &render, layout.traffic_light_center_y);
    let image = finalize_output(&card, &input.config, &render);

    let saved_path = save_image(&image, &input.config)?;
    if input.config.clipboard {
        copy_to_clipboard(&image).unwrap_or_else(|error| {
            eprintln!("Warning: {error}");
        });
    }

    println!("{}", saved_path.display());
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
    use super::{
        apply_card_effects, build_render_config, normalize_index, normalize_span_range,
        render_scale, save_generated_png, Config, PreparedLine, PreparedSegment,
    };
    use image::{Rgba, RgbaImage};

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
            traffic_lights: false,
            line_numbers: false,
            start_line: 1,
            border_radius: 5,
            outer_background: "#ffffff".to_string(),
            outer_padding: 15,
        }
    }

    #[test]
    fn traffic_lights_are_off_when_omitted() {
        let config: Config = serde_json::from_str("{}").unwrap();
        assert!(!config.traffic_lights);
    }

    #[test]
    fn traffic_lights_draw_three_scaled_dots_above_the_code() {
        let mut config = base_config();
        config.scale = 1.5;
        config.traffic_lights = true;
        let render = build_render_config(&config).unwrap();
        let card = RgbaImage::from_pixel(120, 90, render.bg_color);
        let card = apply_card_effects(card, &config, &render, 24.0);

        assert_eq!(render.scaled_title_bar_height, 48);
        assert_eq!(*card.get_pixel(30, 24), Rgba([255, 95, 87, 255]));
        assert_eq!(*card.get_pixel(60, 24), Rgba([254, 188, 46, 255]));
        assert_eq!(*card.get_pixel(90, 24), Rgba([40, 200, 64, 255]));
        assert_eq!(*card.get_pixel(30, 48), render.bg_color);
    }

    #[test]
    fn generated_paths_preserve_existing_images() {
        let directory = std::env::temp_dir().join(format!(
            "snapshot-generator-collision-{}",
            std::process::id()
        ));
        let image = RgbaImage::from_pixel(2, 2, Rgba([1, 2, 3, 255]));
        let first = save_generated_png(&image, &directory, "same-second").unwrap();
        let second_image = RgbaImage::from_pixel(2, 2, Rgba([3, 2, 1, 255]));
        let second = save_generated_png(&second_image, &directory, "same-second").unwrap();
        let first_pixel = image::open(&first).unwrap().to_rgba8().get_pixel(0, 0).0;
        std::fs::remove_dir_all(directory).unwrap();

        assert_eq!(first.file_name().unwrap(), "snapshot-same-second.png");
        assert_eq!(second.file_name().unwrap(), "snapshot-same-second-1.png");
        assert_eq!(first_pixel, [1, 2, 3, 255]);
    }

    #[test]
    fn invalid_scale_uses_fallback() {
        assert_eq!(render_scale(0.0), 2.0);
        assert_eq!(render_scale(-1.0), 2.0);
        assert_eq!(render_scale(1.5), 1.5);
    }

    #[test]
    fn prepared_line_keeps_fractional_segment_advances() {
        let line = PreparedLine {
            segments: vec![
                PreparedSegment {
                    text: "x",
                    width: 7.25,
                    color: Rgba([0, 0, 0, 255]),
                },
                PreparedSegment {
                    text: "y",
                    width: 7.25,
                    color: Rgba([255, 0, 0, 255]),
                },
            ],
        };
        assert_eq!(line.width().ceil(), 15.0);
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
