-- main module file
local module = require("snapshot.module")

---@class Config
---@field output_path string? Path to save the snapshot (defaults to ~/snapshot.png)
---@field padding number? Padding around the code (default: 80)
---@field line_height number? Height of each line in pixels (default: 28)
---@field font_size number? Font size in pixels (default: 20)
---@field background string? Background color in hex format (default: #282c34)
---@field shadow boolean? Enable shadow effect (default: true)
---@field line_numbers boolean? Show line numbers (default: false)
---@field start_line number? Starting line number (default: 1)
local config = {
  output_path = nil,
  padding = 80,
  line_height = 28,
  font_size = 20,
  background = "#282c34",
  shadow = true,
  line_numbers = false,
  start_line = 1,
}

---@class MyModule
local M = {}

---@type Config
M.config = config

---@param args Config?
-- you can define your setup function here. Usually configurations can be merged, accepting outside params and
-- you can also put some validation here for those.
M.setup = function(args)
  M.config = vim.tbl_deep_extend("force", M.config, args or {})
end

M.hello = function()
  return module.my_first_function(M.config.opt)
end

M.capture_buffer = function()
  local buf_lines = vim.api.nvim_buf_get_lines(0, 0, -1, false)
  return buf_lines or {}
end

M.capture_visual = function()
  local start_pos = vim.fn.getpos("'<")
  local end_pos = vim.fn.getpos("'>")
  local buf_lines = vim.api.nvim_buf_get_lines(0, start_pos[2] - 1, end_pos[2], false)

  if #buf_lines == 1 then
    buf_lines[1] = string.sub(buf_lines[1], start_pos[3], end_pos[3])
  else
    buf_lines[1] = string.sub(buf_lines[1], start_pos[3])
    buf_lines[#buf_lines] = string.sub(buf_lines[#buf_lines], 1, end_pos[3])
  end

  return buf_lines or {}
end

M.capture_highlights = function(bufnr)
  bufnr = bufnr or 0
  local extmarks = vim.api.nvim_buf_get_extmarks(bufnr, -1, 0, -1, { details = true })
  return extmarks
end

-- Generate a snapshot image from the current buffer or visual selection
---@param opts table? Optional configuration overrides
M.snapshot = function(opts)
  opts = opts or {}
  local bufnr = vim.api.nvim_get_current_buf()
  
  -- Determine if we're in visual mode or should capture the whole buffer
  local mode = vim.fn.mode()
  local lines
  local start_line_num = 1
  
  if mode == "v" or mode == "V" or mode == "\22" then
    local start_pos = vim.fn.getpos("'<")
    local end_pos = vim.fn.getpos("'>")
    start_line_num = start_pos[2]
    lines = vim.api.nvim_buf_get_lines(bufnr, start_pos[2] - 1, end_pos[2], false)
    
    -- Handle character-wise visual selection
    if mode == "v" and #lines == 1 then
      lines[1] = string.sub(lines[1], start_pos[3], end_pos[3])
    elseif mode == "v" and #lines > 1 then
      lines[1] = string.sub(lines[1], start_pos[3])
      lines[#lines] = string.sub(lines[#lines], 1, end_pos[3])
    end
  else
    lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
  end
  
  -- Build the JSON payload
  local buffer_json = require("snapshot.json").build_snapshot_json(bufnr, lines)
  
  -- Merge config with opts
  local final_config = vim.tbl_deep_extend("force", M.config, opts)
  final_config.start_line = final_config.start_line or start_line_num
  
  -- Set default output path if not provided
  if not final_config.output_path then
    local home = os.getenv("HOME") or "."
    final_config.output_path = home .. "/snapshot.png"
  end
  
  local payload = {
    lines = buffer_json,
    config = final_config,
  }
  
  local json_string = vim.fn.json_encode(payload)
  
  -- Find the generator binary
  local script_path = debug.getinfo(1, "S").source:sub(2)
  local plugin_root = vim.fn.fnamemodify(script_path, ":h:h:h")
  local generator_path = plugin_root .. "/generator/target/release/snapshot-generator"
  
  -- Check if the generator exists
  if vim.fn.executable(generator_path) ~= 1 then
    vim.notify("Snapshot generator not found. Please run: cd " .. plugin_root .. "/generator && cargo build --release", vim.log.levels.ERROR)
    return nil
  end
  
  -- Run the generator
  local cmd = string.format("echo '%s' | %s", json_string:gsub("'", "'\\''"), generator_path)
  local output = vim.fn.system(cmd)
  
  if vim.v.shell_error ~= 0 then
    vim.notify("Failed to generate snapshot: " .. output, vim.log.levels.ERROR)
    return nil
  end
  
  local output_path = output:gsub("%s+", "")
  vim.notify("Snapshot saved to: " .. output_path, vim.log.levels.INFO)
  return output_path
end

return M
