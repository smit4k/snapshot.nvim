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
  "yourusername/snapshot.nvim",
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
  'yourusername/snapshot.nvim',
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
- `:SnapshotVisual` - Create a snapshot of the current visual selection

### Keybindings

Add these to your Neovim config for quick access:

```lua
-- Visual mode: press <leader>cs to capture selection
vim.keymap.set('v', '<leader>cs', ':Snapshot<CR>', { desc = 'Capture snapshot' })

-- Normal mode: capture entire buffer
vim.keymap.set('n', '<leader>cs', ':Snapshot<CR>', { desc = 'Capture snapshot' })
```

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

### Programmatic Usage

You can also call the snapshot function directly from Lua:

```lua
-- With custom options
require("snapshot").snapshot({
  output_path = "/tmp/my-code.png",
  line_numbers = true,
  background = "#ffffff",
})
```

## How It Works

1. **Lua Frontend**: The Neovim plugin captures your code and extracts syntax highlighting information using Treesitter
2. **JSON Communication**: The code and highlighting data are serialized to JSON
3. **Rust Backend**: A Rust binary generates a beautiful PNG image with proper text rendering and colors
4. **Output**: The image is saved to your specified location

## Customization

### Background Colors

Popular color scheme backgrounds:

```lua
{
  background = "#282c34",  -- One Dark (default)
  background = "#1e1e1e",  -- VS Code Dark
  background = "#2e3440",  -- Nord
  background = "#1d2021",  -- Gruvbox
  background = "#011627",  -- Night Owl
}
```

### Font

The plugin uses FiraCode by default. To use a different font:

1. Replace `generator/fonts/FiraCode-Regular.ttf` with your preferred font
2. Rebuild: `cd generator && cargo build --release`

## Troubleshooting

### Generator not found error

If you see "Snapshot generator not found", build the Rust binary:

```bash
cd generator
cargo build --release
```

### Image quality issues

Try adjusting these settings:
- Increase `font_size` for larger text
- Increase `line_height` for better readability
- Adjust `padding` for more space around the code

### Colors don't match my theme

The plugin uses Treesitter highlights. Make sure you have Treesitter parsers installed:

```lua
:TSInstall <your-language>
```

## Development

### Building from source

```bash
git clone https://github.com/yourusername/snapshot.nvim
cd snapshot.nvim/generator
cargo build --release
```

### Running tests

```bash
make test
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - see LICENSE file for details

## Acknowledgments

- Inspired by [Polacode](https://github.com/octref/polacode) for VS Code
- Built with [imageproc](https://github.com/image-rs/imageproc) and [ab_glyph](https://github.com/alexheretic/ab-glyph)
- Uses [FiraCode](https://github.com/tonsky/FiraCode) font
$ gh repo create my-plugin -p ellisonleao/nvim-plugin-template
```

Via github web page:

Click on `Use this template`

![](https://docs.github.com/assets/cb-36544/images/help/repository/use-this-template-button.png)

## Features and structure

- 100% Lua
- Github actions for:
  - running tests using [plenary.nvim](https://github.com/nvim-lua/plenary.nvim) and [busted](https://olivinelabs.com/busted/)
  - check for formatting errors (Stylua)
  - vimdocs autogeneration from README.md file
  - luarocks release (LUAROCKS_API_KEY secret configuration required)

### Plugin structure

```
.
├── lua
│   ├── plugin_name
│   │   └── module.lua
│   └── plugin_name.lua
├── Makefile
├── plugin
│   └── plugin_name.lua
├── README.md
├── tests
│   ├── minimal_init.lua
│   └── plugin_name
│       └── plugin_name_spec.lua
```
