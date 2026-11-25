<h1 align='center'>
    snapshot.nvim
</h1>
<p align='center'>
  <b>Create beautiful screenshots of your code, directly inside Neovim</b>
</p>
    <p align='center'>Inspired by VSCode's <a href="https://github.com/octref/polacode">Polacode Extension</a></p>

## Features

- 📸 Code screenshots with syntax highlighting
- 🎨 Customizable settings (line numbers, padding, shadow, render scale, etc.)
- ⚡️ Blazingly fast Rust renderer

## Installation

**Prerequisites**: Neovim >= 0.9.0

Use your favorite plugin manager!

Ex: using lazy.nvim

```lua
return {
  "smit4k/snapshot.nvim",
  config = function()
    require("snapshot").setup({
        -- Optional config settings, defaults shown below
        snapshot_dir = "~/Pictures/snapshots", -- Reccomended: directory where snapshot images are saved to
        --padding = 25,
        --line_height = 28,
        --font_size = 24,
        --shadow = true,
        --traffic_lights = false, -- set to true for macOS-style window controls
        --show_file_path = false, -- show the buffer's relative file path in a header
        --line_numbers = false,
        --start_line = 1, -- override the first captured line number
        --border_radius = 5,
        --outer_background = "#ffffff", -- use #00000000 for transparent
        --outer_padding = 15,
        --clipboard = true,
    })
  end,
}
```

## Usage

Select code in Visual mode and run `:Snapshot`.
To capture an entire buffer, run `:Snapshot` in Normal mode. An explicit line range, such as `:5,10Snapshot`, captures those full lines.

Set `show_file_path = true` in setup or run `:Snapshot {"show_file_path":true}` to show the file path above the code for a named file buffer. The path is relative to Neovim's global working directory when the plugin loads, usually the directory you launched Neovim from. Files on a different Windows drive or network share use their absolute path.
When `traffic_lights` is enabled, the file path appears to the right of the controls in that header.
With `traffic_lights` disabled, a divider separates the path from the code.

Use `snapshot_dir` to choose the directory for timestamped PNGs. Set `output_path` to an explicit image file, an existing directory, or a new directory marked by a trailing slash. JPEG output flattens transparency onto white.

The generator limits each image to 40 million pixels, including padding and shadow.

## Contributing

From the repository root, run the Lua tests with `nvim --clean --headless -i NONE -l tests/run.lua` and the generator tests with `cargo test --locked --manifest-path generator/Cargo.toml`.

Contributions are welcome! Please open an [issue](https://github.com/smit4k/snapshot.nvim/issues) to discuss your ideas or problems or submit a [pull request](https://github.com/smit4k/snapshot.nvim/pulls) with your changes.
