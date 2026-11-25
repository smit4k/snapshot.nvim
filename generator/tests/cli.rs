use image::GenericImageView;
use serde_json::{json, Value};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

struct Fixture {
    directory: PathBuf,
    executable: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "snapshot-generator-cli-{}-{id}",
            std::process::id()
        ));
        fs::create_dir(&directory).unwrap();

        let binary = Path::new(env!("CARGO_BIN_EXE_snapshot-generator"));
        let executable = directory.join(binary.file_name().unwrap());
        fs::copy(binary, &executable).unwrap();
        // The tiny ttf-parser 0.25.1 test font is licensed in fixtures/LICENSE-MIT.
        fs::write(
            directory.join("JetBrainsMono-Regular.ttf"),
            include_bytes!("fixtures/demo.ttf"),
        )
        .unwrap();

        Self {
            directory,
            executable,
        }
    }

    fn run(&self, input: &str) -> Output {
        let mut child = Command::new(&self.executable)
            .current_dir(&self.directory)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        child.wait_with_output().unwrap()
    }

    fn render(&self, input: Value) -> PathBuf {
        let result = self.run(&input.to_string());
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let path = String::from_utf8(result.stdout).unwrap();
        self.directory.join(path.trim_end_matches(['\r', '\n']))
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

#[test]
fn renders_png_with_expected_dimensions_and_colors() {
    let fixture = Fixture::new();
    let path = fixture.render(json!({
        "lines": [{ "text": "A", "spans": [{ "start": 0, "end": 1, "fg": "#ff0000" }] }],
        "config": {
            "scale": 1, "font_size": 20, "line_height": 18, "padding": 2,
            "outer_padding": 3, "border_radius": 0, "shadow": false,
            "background": "#010203", "outer_background": "#08161f",
            "output_path": "with spaces/image.png", "clipboard": false,
        },
    }));
    assert_eq!(path, fixture.directory.join("with spaces/image.png"));

    let rendered = image::open(path).unwrap().to_rgba8();
    assert_eq!(rendered.dimensions(), (18, 28));
    assert_eq!(rendered.get_pixel(0, 0).0, [8, 22, 31, 255]);
    assert_eq!(rendered.get_pixel(4, 4).0, [1, 2, 3, 255]);
    assert!(rendered
        .pixels()
        .any(|p| p[0] > 100 && p[1] < 8 && p[2] < 8));
}

#[test]
fn renders_file_path_in_a_separate_header_above_numbered_code() {
    let fixture = Fixture::new();
    let mut input = json!({
        "lines": [{ "text": "A", "spans": [] }],
        "config": {
            "scale": 1, "font_size": 20, "line_height": 24, "padding": 2,
            "outer_padding": 0, "border_radius": 0, "shadow": false,
            "line_numbers": true, "start_line": 42,
            "background": "#000000", "foreground": "#ffffff",
            "output_path": "code.png", "clipboard": false,
        },
    });
    let code = image::open(fixture.render(input.clone()))
        .unwrap()
        .to_rgba8();
    input["file_path"] = json!("AAAAAAAAAAAA/AAAA.lua");
    input["config"]["output_path"] = json!("with-path.png");
    let with_path = image::open(fixture.render(input)).unwrap().to_rgba8();

    let (width, height) = code.dimensions();
    let header_height = 32;
    let divider_y = header_height - 1;
    assert!(with_path.width() > width);
    assert_eq!(with_path.height(), height + header_height);
    let code_bg = code.get_pixel(0, 0);
    assert_eq!(with_path.get_pixel(0, 0), code_bg);
    let divider_left = (0..with_path.width())
        .find(|&x| with_path.get_pixel(x, divider_y) != code_bg)
        .unwrap();
    assert!(divider_left > 2);
    assert_eq!(
        with_path.get_pixel(with_path.width() - 2, divider_y),
        code_bg
    );
    assert_eq!(with_path.get_pixel(0, divider_y + 1), code_bg);
    let path_left =
        image::imageops::crop_imm(&with_path, 0, 0, with_path.width(), header_height - 1)
            .pixels()
            .filter_map(|(x, _, p)| (p[0] > 100).then_some(x))
            .min()
            .unwrap();
    let code_left = code
        .enumerate_pixels()
        .filter_map(|(x, _, p)| (p[0] > 100).then_some(x))
        .min()
        .unwrap();
    assert!(path_left.abs_diff(divider_left) <= 1);
    assert!(code_left.abs_diff(divider_left) <= 1);
    assert_eq!(
        image::imageops::crop_imm(&with_path, 0, divider_y + 1, width, height).to_image(),
        code,
    );
}

#[test]
fn renders_file_path_beside_the_traffic_lights_in_one_row() {
    let fixture = Fixture::new();
    let path = fixture.render(json!({
        "lines": [{ "text": "A", "spans": [] }],
        "file_path": "/AAAA/AAAA.lua",
        "config": {
            "scale": 1, "font_size": 20, "line_height": 24, "padding": 2,
            "outer_padding": 0, "border_radius": 0, "shadow": false,
            "traffic_lights": true, "background": "#000000", "foreground": "#ffffff",
            "output_path": "header.png", "clipboard": false,
        },
    }));
    let rendered = image::open(path).unwrap().to_rgba8();

    assert_eq!(rendered.height(), 32 + 24 + 4);
    assert_eq!(*rendered.get_pixel(60, 16), image::Rgba([40, 200, 64, 255]));
    assert!(rendered.enumerate_pixels().any(|(x, y, p)| {
        x >= 78 && (10..22).contains(&y) && p[0] > 100 && p[0] == p[1] && p[1] == p[2]
    }));
    assert!(rendered
        .rows()
        .nth(31)
        .unwrap()
        .all(|pixel| pixel.0 == [0, 0, 0, 255]));
}

#[test]
fn output_path_with_trailing_separator_creates_a_png_in_the_directory() {
    let fixture = Fixture::new();
    let path = fixture.render(json!({
        "lines": [{ "text": "A", "spans": [] }],
        "config": { "output_path": "new directory/", "clipboard": false },
    }));

    assert_eq!(
        path.parent().unwrap(),
        fixture.directory.join("new directory")
    );
    assert_eq!(path.extension().unwrap(), "png");
    assert!(image::open(path).is_ok());
}

#[test]
fn renders_transparency_over_white_for_jpeg() {
    let fixture = Fixture::new();
    let path = fixture.render(json!({
        "lines": [{ "text": "", "spans": [] }],
        "config": {
            "output_path": "transparent.jpeg", "shadow": false, "clipboard": false,
            "background": "#ff000000", "outer_background": "#00000000",
        },
    }));

    let rendered = image::open(path).unwrap().to_rgb8();
    assert_eq!(rendered.get_pixel(0, 0).0, [255; 3]);
}

#[test]
fn reports_malformed_json_on_stderr() {
    let result = Fixture::new().run("{ invalid");
    assert!(!result.status.success());
    assert!(result.stdout.is_empty());
    assert!(String::from_utf8_lossy(&result.stderr).contains("Failed to parse JSON input"));
}

#[test]
fn rejects_zero_sized_and_oversized_images_before_saving() {
    let fixture = Fixture::new();
    for (name, value, expected) in [
        ("scale", json!(0.001), "width and height must be positive"),
        ("scale", json!(1e30), "Scaled font size"),
        ("outer_padding", json!(u32::MAX), "40000000-pixel limit"),
        ("padding", json!(u32::MAX), "40000000-pixel limit"),
    ] {
        let mut input = json!({
            "lines": [{ "text": "A", "spans": [] }],
            "config": { "output_path": "image.png", "clipboard": false },
        });
        input["config"][name] = value;
        let result = fixture.run(&input.to_string());

        assert!(!result.status.success());
        let error = String::from_utf8_lossy(&result.stderr);
        assert!(error.contains(expected), "{error}");
        assert!(!fixture.directory.join("image.png").exists());
    }
}
