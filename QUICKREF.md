# Quick Reference

## Commands

| Command | Description |
|---------|-------------|
| `:Snapshot` | Create snapshot of visual selection or entire buffer |
| `:SnapshotDebug` | Print debug info about plugin paths and generator binary |

## Keybindings

Add to your config:

```lua
-- Visual mode: capture selection
vim.keymap.set('v', '<leader>cs', ':Snapshot<CR>', { desc = 'Capture snapshot' })

-- Normal mode: capture entire buffer
vim.keymap.set('n', '<leader>cs', ':Snapshot<CR>', { desc = 'Capture snapshot' })
```

## Configuration Options

```lua
require("snapshot").setup({
  -- Output path for snapshots (default: ~/snapshot.png)
  output_path = "~/Pictures/code.png",
  
  -- Padding around code in pixels (default: 80)
  padding = 100,
  
  -- Line height in pixels (default: 28)
  line_height = 30,
  
  -- Font size in pixels (default: 20)
  font_size = 22,
  
  -- Background color in hex (default: #282c34)
  background = "#1e1e1e",
  
  -- Enable shadow effect (default: true)
  shadow = true,
  
  -- Show line numbers (default: false)
  line_numbers = true,
  
  -- Starting line number (default: 1)
  start_line = 1,
})
```

## Common Backgrounds

```lua
background = "#282c34"  -- One Dark (default)
background = "#1e1e1e"  -- VS Code Dark
background = "#2e3440"  -- Nord
background = "#1d2021"  -- Gruvbox Dark
background = "#011627"  -- Night Owl
background = "#ffffff"  -- Light background
```

## Programmatic Usage

```lua
-- Create snapshot with custom options
require("snapshot").snapshot({
  output_path = "/tmp/code.png",
  line_numbers = true,
  font_size = 24,
})
```

## Workflow

1. **Select code**: Use visual mode (`v`, `V`, or `Ctrl-v`)
2. **Capture**: Run `:Snapshot` or use keybinding
3. **Find image**: Check the output path (default: `~/snapshot.png`)
4. **Share**: Use the generated image in documentation, social media, etc.

## Tips

- Use `line_numbers = true` for code examples
- Increase `padding` for more whitespace
- Adjust `font_size` for better readability
- Change `background` to match your theme
- Create multiple presets with different configs:

```lua
-- Preset for social media
local function snapshot_social()
  require("snapshot").snapshot({
    output_path = "~/Pictures/code-share.png",
    padding = 100,
    font_size = 24,
    line_numbers = false,
  })
end

-- Preset for documentation
local function snapshot_docs()
  require("snapshot").snapshot({
    output_path = "~/docs/code.png",
    padding = 60,
    font_size = 18,
    line_numbers = true,
  })
end

vim.keymap.set('v', '<leader>css', snapshot_social, { desc = 'Snapshot for social' })
vim.keymap.set('v', '<leader>csd', snapshot_docs, { desc = 'Snapshot for docs' })
```
