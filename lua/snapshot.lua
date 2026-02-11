-- main module file
local module = require("snapshot.module")

---@class Config
---@field snapshot_dir string? Directory to save snapshots (defaults to $HOME)
---@field output_path string? Path to save the snapshot (defaults to snapshot_dir/snapshot-{timestamp}.png)
---@field padding number? Padding around the code (default: 80)
---@field line_height number? Height of each line in pixels (default: 28)
---@field font_size number? Font size in pixels (default: 20)
---@field background string? Background color in hex format (default: #282c34)
---@field clipboard boolean? Enable saving snapshot to clipboard
---@field shadow boolean? Enable shadow effect (default: true)
---@field line_numbers boolean? Show line numbers (default: false)
---@field start_line number? Starting line number (default: 1)
local config = {
  padding = 80,
  line_height = 28,
  font_size = 20,
  background = "#282c34",
  clipboard = true,
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

  -- Check if we have a visual selection by checking the marks
  local start_pos = vim.fn.getpos("'<")
  local end_pos = vim.fn.getpos("'>")
  local lines
  local start_line_num = 1

  -- If the visual marks are valid and different, use them
  if start_pos[2] > 0 and end_pos[2] > 0 and start_pos[2] <= end_pos[2] then
    start_line_num = start_pos[2]
    lines = vim.api.nvim_buf_get_lines(bufnr, start_pos[2] - 1, end_pos[2], false)

    -- Handle character-wise visual selection
    -- If it's a single line and columns are specified
    if #lines == 1 and start_pos[3] > 0 and end_pos[3] > 0 then
      lines[1] = string.sub(lines[1], start_pos[3], end_pos[3])
    elseif #lines > 1 and start_pos[3] > 0 and end_pos[3] > 0 then
      -- Multi-line character-wise selection
      lines[1] = string.sub(lines[1], start_pos[3])
      lines[#lines] = string.sub(lines[#lines], 1, end_pos[3])
    end
  else
    -- No valid visual selection, capture the entire buffer
    lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
  end

  -- Build the JSON payload
  local buffer_json = require("snapshot.json").build_snapshot_json(bufnr, lines, start_line_num - 1)

  -- Merge config with opts
  local final_config = vim.tbl_deep_extend("force", M.config, opts)
  final_config.start_line = final_config.start_line or start_line_num

  -- Handle snapshot_dir configuration
  if final_config.snapshot_dir then
    -- Expand tilde and env vars
    final_config.snapshot_dir = vim.fn.expand(final_config.snapshot_dir)
    -- Remove trailing slash
    final_config.snapshot_dir = final_config.snapshot_dir:gsub("/$", "")

    -- Validate that snapshot_dir is or will be a directory
    if vim.fn.isdirectory(final_config.snapshot_dir) == 0 then
      -- Check if it exists as a file (error condition)
      if vim.fn.filereadable(final_config.snapshot_dir) == 1 then
        vim.notify(
          "Error: snapshot_dir '" .. final_config.snapshot_dir .. "' exists but is not a directory.",
          vim.log.levels.ERROR
        )
        return nil
      end
      -- Directory doesn't exist yet, which is fine (Rust will create it)
    end
  end

  -- Normalize user-provided output path (do NOT set a default filename here)
  -- If output_path is nil, we'll remove it from the config so Rust uses its default (timestamped)
  if final_config.output_path then
    -- Expand tilde and env vars
    final_config.output_path = vim.fn.expand(final_config.output_path)

    -- If user passed a directory, let Rust generate the filename
    if vim.fn.isdirectory(final_config.output_path) == 1 then
      final_config.output_path = final_config.output_path:gsub("/$", "")
    elseif final_config.output_path:match("/$") then
      -- Trailing slash but dir may not exist yet: still treat as directory
      final_config.output_path = final_config.output_path:gsub("/$", "")
    end

    -- Optional: warn about weird extensions (only when user set a file)
    if
      not final_config.output_path:match("%.[pP][nN][gG]$")
      and not final_config.output_path:match("%.[jJ][pP][eE]?[gG]$")
      and not final_config.output_path:match("%.[wW][eE][bB][pP]$")
      and vim.fn.isdirectory(final_config.output_path) == 0
    then
      vim.notify(
        "Warning: output_path '"
          .. final_config.output_path
          .. "' may not have a valid image extension (.png, .jpg, .jpeg, .webp).",
        vim.log.levels.WARN
      )
    end
  end

  -- Build payload, excluding output_path/snapshot_dir if they're nil so Rust uses its default
  local config_for_json = {}
  for k, v in pairs(final_config) do
    if (k ~= "output_path" and k ~= "snapshot_dir") or v ~= nil then
      config_for_json[k] = v
    end
  end

  local payload = {
    lines = buffer_json,
    config = config_for_json,
  }

  local json_string = vim.fn.json_encode(payload)

  -- Find the generator binary using multiple methods for reliability
  local generator_path

  -- Method 1: Try to find via the loaded module path
  local snapshot_module = package.loaded["snapshot"]
  if snapshot_module and snapshot_module.__file then
    local module_path = snapshot_module.__file
    local plugin_root = vim.fn.fnamemodify(module_path, ":h:h")
    generator_path = plugin_root .. "/generator/target/release/snapshot-generator"
  end

  -- Method 2: Use runtimepath to find the plugin
  if not generator_path or vim.fn.executable(generator_path) ~= 1 then
    local rtp = vim.api.nvim_list_runtime_paths()
    for _, path in ipairs(rtp) do
      if path:match("snapshot%.nvim") or path:match("snapshot$") then
        local test_path = path .. "/generator/target/release/snapshot-generator"
        if vim.fn.executable(test_path) == 1 then
          generator_path = test_path
          break
        end
      end
    end
  end

  -- Check if the generator exists
  if not generator_path or vim.fn.executable(generator_path) ~= 1 then
    -- Try to provide helpful error message with correct path
    local rtp = vim.api.nvim_list_runtime_paths()
    local plugin_path = nil
    for _, path in ipairs(rtp) do
      if path:match("snapshot%.nvim") or path:match("snapshot$") then
        plugin_path = path
        break
      end
    end

    if plugin_path then
      vim.notify(
        "Snapshot generator not found. Please run:\ncd " .. plugin_path .. "/generator && cargo build --release",
        vim.log.levels.ERROR
      )
    else
      vim.notify(
        "Snapshot generator not found and plugin path could not be determined.\nPlease build the generator manually.",
        vim.log.levels.ERROR
      )
    end
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
