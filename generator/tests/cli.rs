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
