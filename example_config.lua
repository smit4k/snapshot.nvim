-- Example configuration for snapshot.nvim
-- Add this to your Neovim config (init.lua or lazy plugin spec)

return {
  "yourusername/snapshot.nvim",
  build = "cd generator && cargo build --release",
  
  -- Lazy load on command or keymap
  cmd = { "Snapshot", "SnapshotDebug" },
  keys = {
    { "<leader>cs", mode = { "n", "v" }, desc = "Code Snapshot" },
  },
  
  config = function()
    local snapshot = require("snapshot")
    
    -- Basic setup with defaults
    snapshot.setup({
      output_path = nil,        -- defaults to ~/snapshot.png
      padding = 80,             -- pixels of padding around code
      line_height = 28,         -- line height in pixels
      font_size = 20,           -- font size in pixels
      background = "#282c34",   -- background color (One Dark theme)
      shadow = true,            -- enable shadow effect (experimental)
      line_numbers = false,     -- show line numbers
    })
    
    -- Keybindings
    vim.keymap.set("v", "<leader>cs", ":Snapshot<CR>", { 
      desc = "Capture code snapshot",
      silent = true,
    })
    
    vim.keymap.set("n", "<leader>cs", ":Snapshot<CR>", { 
      desc = "Capture buffer snapshot",
      silent = true,
    })
    
    -- Custom functions for different use cases
    
    -- Snapshot for social media (larger, no line numbers)
    vim.keymap.set("v", "<leader>css", function()
      snapshot.snapshot({
        output_path = vim.fn.expand("~/Pictures/code-share.png"),
        padding = 100,
        font_size = 24,
        line_numbers = false,
      })
    end, { desc = "Snapshot for social media" })
    
    -- Snapshot for documentation (with line numbers)
    vim.keymap.set("v", "<leader>csd", function()
      snapshot.snapshot({
        output_path = vim.fn.expand("~/docs/code-example.png"),
        padding = 60,
        font_size = 18,
        line_numbers = true,
      })
    end, { desc = "Snapshot for docs" })
    
    -- Snapshot with custom background
    vim.keymap.set("v", "<leader>csw", function()
      snapshot.snapshot({
        output_path = vim.fn.expand("~/snapshot-light.png"),
        background = "#ffffff",
        font_size = 20,
      })
    end, { desc = "Snapshot with white background" })
  end,
}

-- Alternative: Minimal setup
-- require("snapshot").setup()

-- Alternative: Custom setup with common themes
--[[
require("snapshot").setup({
  -- VS Code Dark theme
  background = "#1e1e1e",
  
  -- Nord theme
  -- background = "#2e3440",
  
  -- Gruvbox Dark theme
  -- background = "#1d2021",
  
  -- Night Owl theme
  -- background = "#011627",
  
  -- Light theme
  -- background = "#ffffff",
  
  -- Tokyo Night theme
  -- background = "#1a1b26",
  
  -- Dracula theme
  -- background = "#282a36",
})
--]]
