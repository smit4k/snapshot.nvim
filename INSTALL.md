# Installation Guide

## Prerequisites

Before installing snapshot.nvim, make sure you have:

1. **Neovim >= 0.9.0**
   ```bash
   nvim --version
   ```

2. **Rust and Cargo**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. **Treesitter** (for syntax highlighting)
   Install parsers for your languages:
   ```vim
   :TSInstall lua javascript python rust
   ```

## Installation

### Using lazy.nvim (Recommended)

Add this to your Neovim config:

```lua
return {
  "yourusername/snapshot.nvim",
  build = "cd generator && cargo build --release",
  keys = {
    { "<leader>cs", ":Snapshot<CR>", mode = { "n", "v" }, desc = "Code Snapshot" },
  },
  config = function()
    require("snapshot").setup({
      output_path = nil,        -- defaults to ~/snapshot.png
      padding = 80,
      line_height = 28,
      font_size = 20,
      background = "#282c34",
      line_numbers = false,
    })
  end,
}
```

### Using packer.nvim

```lua
use {
  'yourusername/snapshot.nvim',
  run = 'cd generator && cargo build --release',
  config = function()
    require("snapshot").setup()
  end
}
```

### Manual Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/snapshot.nvim ~/.config/nvim/pack/plugins/start/snapshot.nvim
   ```

2. Build the generator:
   ```bash
   cd ~/.config/nvim/pack/plugins/start/snapshot.nvim/generator
   cargo build --release
   ```

3. Add to your init.lua:
   ```lua
   require("snapshot").setup()
   ```

## Verification

Test that everything works:

1. Open a file with some code
2. Select a few lines in visual mode (`V`)
3. Run `:Snapshot`
4. Check that `~/snapshot.png` was created

## Troubleshooting

### "Snapshot generator not found"

The Rust binary wasn't built. Run:
```bash
cd ~/.config/nvim/pack/plugins/start/snapshot.nvim/generator
cargo build --release
```

### Colors not showing

Make sure you have Treesitter installed and parsers for your language:
```vim
:TSInstall <language>
```

### Font issues

The plugin uses FiraCode by default. If you want to use a different font:
1. Replace `generator/fonts/FiraCode-Regular.ttf`
2. Rebuild: `cargo build --release`

## Next Steps

- Read the [README](README.md) for usage examples
- Check out [EXAMPLES.md](EXAMPLES.md) for more examples
- Customize your config (see README for options)
