# snapshot.nvim

![Lua](https://img.shields.io/badge/Made%20with%20Lua-blueviolet.svg?style=for-the-badge&logo=lua)
![Rust](https://img.shields.io/badge/Powered%20by%20Rust-orange.svg?style=for-the-badge&logo=rust)

A beautiful Neovim plugin for creating code snapshots with syntax highlighting, inspired by VSCode's Polacode extension.

## Features

- 📸 Create beautiful code screenshots with syntax highlighting
- 🎨 Fully customizable appearance (colors, fonts, padding, etc.)
- 🔢 Optional line numbers
- 🌈 Preserves your Neovim color scheme
- ⚡ Fast Rust-powered image generation
- 🎯 Works with visual selections or entire buffers

## Installation

### Prerequisites

- Neovim >= 0.9.0
- Rust and Cargo (for building the image generator)

### Using [lazy.nvim](https://github.com/folke/lazy.nvim)

```lua
{
  "smit4k/snapshot.nvim",
  build = "cd generator && cargo build --release",
  config = function()
    require("snapshot").setup({
      -- Optional configuration
      output_path = nil,        -- defaults to ~/snapshot.png
      padding = 80,             -- padding around code in pixels
      line_height = 28,         -- line height in pixels
      font_size = 20,           -- font size in pixels
      background = "#282c34",   -- background color (hex)
      shadow = true,            -- enable shadow (experimental)
      line_numbers = false,     -- show line numbers
    })
  end,
}
```

### Using [packer.nvim](https://github.com/wbthomason/packer.nvim)

```lua
use {
  'smit4k/snapshot.nvim',
  run = 'cd generator && cargo build --release',
  config = function()
    require("snapshot").setup({
      -- your config here
    })
  end
}
```

## Usage

### Basic Usage

1. **Capture a visual selection:**
   - Select some code in visual mode (`v`, `V`, or `Ctrl-v`)
   - Run `:Snapshot`
   - The snapshot will be saved to `~/snapshot.png` by default

2. **Capture the entire buffer:**
   - In normal mode, run `:Snapshot`
   - The entire buffer will be captured

### Commands

- `:Snapshot` - Create a snapshot of visual selection or entire buffer

### Configuration

You can customize the appearance of your snapshots:

```lua
require("snapshot").setup({
  output_path = "~/Pictures/code-snapshot.png",  -- Custom output path
  padding = 100,                                  -- More padding
  line_height = 30,                              -- Taller lines
  font_size = 22,                                -- Larger font
  background = "#1e1e1e",                        -- VS Code dark background
  line_numbers = true,                           -- Show line numbers
})
```

### Font

The plugin uses FiraCode by default. To use a different font:

1. Replace `generator/fonts/FiraCode-Regular.ttf` with your preferred font
2. Rebuild: `cd generator && cargo build --release`
