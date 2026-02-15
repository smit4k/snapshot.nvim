<h1 align='center'>
    snapshot.nvim
</h1>
<p align='center'>
  <b>Create beautiful screenshots of your code, directly inside Neovim</b>
</p>
    <p align='center'>Inspired by VSCode's <a href="https://github.com/octref/polacode">Polacode Extension</a></p>

## Features

- 📸 Code snapshots with syntax highlighting
- 🎨 Customizable settings (line numbers, padding, shadow, render scale, etc.)
- ⚡️ Blazingly fast Rust renderer

## Installation

**Prerequisites**: Neovim >= 0.9.0, [Rust & Cargo](https://rust-lang.org/tools/install/)

Use your favorite plugin manager!

Ex: using lazy.nvim

```lua
return {
  "smit4k/snapshot.nvim",
  build = "cd generator && cargo build --release",
  cmd = { "Snapshot", "SnapshotDebug" },
  config = function()
    require("snapshot").setup({
        snapshot_dir = "~/Pictures/snapshots", -- Directory where snapshot images are saved to (optional)
        padding = 80,
        line_height = 28,
        font_size = 20,
        background = "#282c34",
        shadow = true,
        line_numbers = false,
        start_line = 1,
    })
  end,
}
```

## Usage

Select code in Visual mode and run `:Snapshot`.

## Contributing

Contributions are welcome! Please open an [issue](https://github.com/smit4k/snapshot.nvim/issues) to discuss your ideas or problems or submit a [pull request](https://github.com/smit4k/snapshot.nvim/pulls) with your changes.
