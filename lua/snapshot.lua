-- main module file
local module = require("snapshot.module")
local highlights = require("snapshot.highlights")

local is_windows = vim.fn.has("win32") == 1 or vim.fn.has("win64") == 1
local generator_bin = is_windows and "snapshot-generator.exe" or "snapshot-generator"

--- Resolve the Normal highlight group to get the editor's background and foreground colors.
--- Falls back to sensible defaults if the highlight group is not set.
local function get_editor_colors()
  local normal = highlights.resolve_hl("Normal")
  return {
    bg = normal.bg or "#282c34",
    fg = normal.fg or "#abb2bf",
  }
end

---@class Config
---@field snapshot_dir string? Directory to save snapshots (defaults to $HOME)
---@field output_path string? Image file path or a directory for a generated PNG
---@field scale number? Resolution scale factor for HiDPI/Retina rendering (default: 2)
---@field padding number? Padding around the code (default: 80)
---@field line_height number? Height of each line in pixels (default: 28)
---@field font_size number? Font size in pixels (default: 20)
---@field background string? Background color in hex format (default: from Normal highlight group)
---@field foreground string? Default text color in hex format (default: from Normal highlight group)
---@field clipboard boolean? Enable saving snapshot to clipboard
---@field shadow boolean? Enable shadow effect (default: true)
---@field traffic_lights boolean? Show macOS-style window controls above the code (default: false)
---@field line_numbers boolean? Show line numbers (default: false)
---@field start_line number? Starting line number (default: first captured line)
---@field border_radius number? Corner radius in pixels (default: 5)
---@field outer_background string? Background color outside the code card in hex format (default: "#ffffff")
---@field outer_padding number? Padding between the code card and outer background edge in pixels (default: 40)
local config = {
  scale = 2,
  padding = 25,
  line_height = 28,
  font_size = 24,
  clipboard = true,
  shadow = true,
  traffic_lights = false,
  line_numbers = false,
  border_radius = 5,
  outer_background = "#ffffff",
  outer_padding = 15,
}

---@class MyModule
local M = {}

---@type Config
M.config = config

local VERSION = "v0.1.5"
local FONT_NAME = "JetBrainsMono-Regular.ttf"

--- Read the installed version from the .version file in the release directory.
--- Returns nil if the file doesn't exist or can't be read.
local function read_installed_version(plugin_root)
  local version_file = plugin_root .. "/generator/target/release/.version"
  local f = io.open(version_file, "r")
  if not f then
    return nil
  end
  local version = f:read("*l")
  f:close()
  return version
end

-- you can define your setup function here. Usually configurations can be merged, accepting outside params and
-- you can also put some validation here for those.
M.setup = function(args)
  M.config = vim.tbl_deep_extend("force", M.config, args or {})

  -- Auto-install or update the generator and its font if needed
  local rtp = vim.api.nvim_list_runtime_paths()
  for _, path in ipairs(rtp) do
    if path:match("snapshot%.nvim") or path:match("snapshot$") then
      local release_dir = path .. "/generator/target/release/"
      if vim.fn.executable(release_dir .. generator_bin) ~= 1 or vim.fn.getfsize(release_dir .. FONT_NAME) <= 0 then
        vim.notify("snapshot.nvim: binary or font not found, installing...", vim.log.levels.INFO)
        M.install()
      else
        local installed_version = read_installed_version(path)
        if installed_version ~= VERSION then
          vim.notify(
            "snapshot.nvim: updating binary from " .. (installed_version or "unknown") .. " to " .. VERSION .. "...",
            vim.log.levels.INFO
          )
          M.install()
        end
      end
      break
    end
  end
end

local function run_install_command(args)
  local ok, output = pcall(vim.fn.system, args)
  return ok and vim.v.shell_error == 0, output
end

M.install = function()
  local os_name = jit.os:lower() -- "linux", "osx", "windows"
  local arch = jit.arch -- "x64", "arm64"

  -- Map to GitHub release artifact names
  local filename
  if os_name == "linux" and arch == "x64" then
    filename = "snapshot-generator-linux-x86_64"
  elseif os_name == "osx" and arch == "arm64" then
    filename = "snapshot-generator-macos-aarch64"
  elseif os_name == "osx" and arch == "x64" then
    filename = "snapshot-generator-macos-x86_64"
  elseif os_name == "windows" then
    filename = "snapshot-generator-windows-x86_64.exe"
  else
    vim.notify("Unsupported platform: " .. os_name .. " " .. arch, vim.log.levels.ERROR)
    return false
  end

  -- Find plugin root path
  local plugin_root = nil
  local rtp = vim.api.nvim_list_runtime_paths()
  for _, path in ipairs(rtp) do
    if path:match("snapshot%.nvim") or path:match("snapshot$") then
      plugin_root = path
      break
    end
  end

  if not plugin_root then
    vim.notify("Could not find snapshot.nvim plugin path", vim.log.levels.ERROR)
    return false
  end

  local generator_dir = plugin_root .. "/generator/"
  local target_dir = generator_dir .. "target"
  local dest_dir = target_dir .. "/release/"
  local dest = dest_dir .. generator_bin
  vim.fn.mkdir(dest_dir, "p")

  local url = "https://github.com/smit4k/snapshot.nvim/releases/download/" .. VERSION .. "/" .. filename
  local download = dest .. ".download-" .. vim.fn.getpid()
  vim.notify("Downloading snapshot generator...", vim.log.levels.INFO)
  local downloaded = run_install_command({ "curl", "-fLo", download, url })
  if downloaded and not is_windows then
    downloaded = run_install_command({ "chmod", "+x", download })
  end

  if not downloaded or vim.fn.rename(download, dest) ~= 0 then
    vim.fn.delete(download)
    vim.notify("Download failed, falling back to cargo build...", vim.log.levels.WARN)
    local built, output = run_install_command({
      "cargo",
      "build",
      "--release",
      "--manifest-path",
      generator_dir .. "Cargo.toml",
      "--target-dir",
      target_dir,
    })
    if not built then
      vim.notify("Cargo build failed: " .. output, vim.log.levels.ERROR)
      return false
    end
  end

  if vim.fn.executable(dest) ~= 1 then
    vim.notify("Snapshot generator is missing or not executable: " .. dest, vim.log.levels.ERROR)
    return false
  end

  if vim.fn.getfsize(dest_dir .. FONT_NAME) <= 0 then
    vim.notify("Downloading font...", vim.log.levels.INFO)
    local font_url = "https://github.com/JetBrains/JetBrainsMono/releases/download/v2.304/JetBrainsMono-2.304.zip"
    local font_zip = vim.fn.tempname() .. ".zip"
    local font_downloaded, output = run_install_command({ "curl", "-fLo", font_zip, font_url })
    if not font_downloaded then
      vim.fn.delete(font_zip)
      vim.notify("Font download failed: " .. output, vim.log.levels.ERROR)
      return false
    end

    local extracted
    if is_windows then
      local extract_dir = vim.fn.tempname()
      local ps_script = table.concat({
        "$ErrorActionPreference='Stop'",
        "$zip='" .. font_zip:gsub("'", "''") .. "'",
        "$tmp='" .. extract_dir:gsub("'", "''") .. "'",
        "$dest='" .. dest_dir:gsub("'", "''") .. "'",
        "Expand-Archive -LiteralPath $zip -DestinationPath $tmp -Force",
        "try {",
        "$font=Get-ChildItem -LiteralPath $tmp -Recurse -Filter '" .. FONT_NAME .. "' | Select-Object -First 1",
        "if (-not $font) { throw 'Font not found in archive' }",
        "Copy-Item -LiteralPath $font.FullName -Destination (Join-Path $dest '" .. FONT_NAME .. "') -Force",
        "} finally { Remove-Item -LiteralPath $tmp -Recurse -Force }",
      }, "\n")
      extracted, output = run_install_command({ "powershell", "-NoProfile", "-Command", ps_script })
    else
      extracted, output =
        run_install_command({ "unzip", "-j", "-o", font_zip, "fonts/ttf/" .. FONT_NAME, "-d", dest_dir })
    end
    vim.fn.delete(font_zip)

    if not extracted or vim.fn.getfsize(dest_dir .. FONT_NAME) <= 0 then
      vim.fn.delete(dest_dir .. FONT_NAME)
      vim.notify("Font extraction failed: " .. output, vim.log.levels.ERROR)
      return false
    end
  end

  local written, status = pcall(vim.fn.writefile, { VERSION }, dest_dir .. ".version")
  if not written or status ~= 0 then
    vim.notify("Failed to write generator version: " .. tostring(status), vim.log.levels.ERROR)
    return false
  end

  vim.notify("Snapshot generator installed successfully!", vim.log.levels.INFO)
  return true
end

M.hello = function()
  return module.my_first_function(M.config.opt)
end

M.capture_highlights = function(bufnr)
  bufnr = bufnr or 0
  local extmarks = vim.api.nvim_buf_get_extmarks(bufnr, -1, 0, -1, { details = true })
  return extmarks
end

local function crop_visual_lines(lines, first, last, mode)
  local exclusive = vim.o.selection == "exclusive"
  local left_col, right_col
  if mode == "\22" then
    local first_col, last_col = vim.fn.virtcol("'<"), vim.fn.virtcol("'>")
    left_col, right_col = math.min(first_col, last_col), math.max(first_col, last_col)
    if exclusive and left_col < right_col then
      right_col = right_col - 1
    end
  end

  for i, line in ipairs(lines) do
    local start_col, end_col = 0, #line.text
    if mode == "v" then
      if i == 1 then
        start_col = first[3] - 1
      end
      if i == #lines then
        local last_col = last[3]
        if exclusive and (first[2] ~= last[2] or first[3] ~= last_col) then
          end_col = last_col - 1
        elseif last_col <= #line.text then
          end_col = last_col + vim.str_utf_end(line.text, last_col)
        end
      end
    elseif mode == "\22" then
      -- shortcut: a selected cell within a tab or wide glyph keeps that glyph, until the renderer supports partial cells.
      local row = first[2] + i - 1
      if left_col > vim.fn.strdisplaywidth(line.text) then
        start_col, end_col = #line.text, #line.text
      else
        start_col = vim.fn.virtcol2col(0, row, left_col) - 1
        local last_col = vim.fn.virtcol2col(0, row, right_col)
        end_col = last_col + vim.str_utf_end(line.text, last_col)
      end
    end

    line.text = line.text:sub(start_col + 1, end_col)
    local spans = {}
    for _, span in ipairs(line.spans) do
      local start = math.max(span.start, start_col)
      local finish = math.min(span["end"], end_col)
      if start < finish then
        span.start = start - start_col
        span["end"] = finish - start_col
        spans[#spans + 1] = span
      end
    end
    line.spans = spans
  end
end

--- Generate a snapshot image from the current buffer or a command range.
---@param opts table? Optional configuration overrides
---@param range table? Command line range and optional Visual selection
M.snapshot = function(opts, range)
  opts = opts or {}
  local bufnr = vim.api.nvim_get_current_buf()
  local start_line_num = range and range.line1 or 1
  local lines = vim.api.nvim_buf_get_lines(bufnr, start_line_num - 1, range and range.line2 or -1, false)
  local buffer_json = require("snapshot.json").build_snapshot_json(bufnr, lines, start_line_num - 1)
  if range and range.visual then
    local mode = vim.fn.visualmode()
    local first, last = vim.fn.getpos("'<"), vim.fn.getpos("'>")
    if first[2] == range.line1 and last[2] == range.line2 and (mode == "v" or mode == "\22") then
      crop_visual_lines(buffer_json, first, last, mode)
    end
  end

  -- Merge config with opts
  local final_config = vim.tbl_deep_extend("force", M.config, opts)
  final_config.start_line = final_config.start_line or start_line_num

  -- Resolve background and foreground from the editor's Normal highlight group
  -- if the user hasn't explicitly configured them.
  local editor_colors = get_editor_colors()
  if not final_config.background then
    final_config.background = editor_colors.bg
  end
  if not final_config.foreground then
    final_config.foreground = editor_colors.fg
  end

  local payload = {
    lines = buffer_json,
    config = final_config,
  }

  local json_string = vim.fn.json_encode(payload)

  -- Find the generator binary using multiple methods for reliability
  local generator_path

  -- Method 1: Try to find via the loaded module path
  local snapshot_module = package.loaded["snapshot"]
  if snapshot_module and snapshot_module.__file then
    local module_path = snapshot_module.__file
    local plugin_root = vim.fn.fnamemodify(module_path, ":h:h")
    generator_path = plugin_root .. "/generator/target/release/" .. generator_bin
  end

  -- Method 2: Use runtimepath to find the plugin
  if not generator_path or vim.fn.executable(generator_path) ~= 1 then
    local rtp = vim.api.nvim_list_runtime_paths()
    for _, path in ipairs(rtp) do
      if path:match("snapshot%.nvim") or path:match("snapshot$") then
        local test_path = path .. "/generator/target/release/" .. generator_bin
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

  local output, errors = "", ""
  local started, job = pcall(vim.fn.jobstart, { generator_path }, {
    stdout_buffered = true,
    stderr_buffered = true,
    on_stdout = function(_, data)
      output = table.concat(data, "\n")
    end,
    on_stderr = function(_, data)
      errors = table.concat(data, "\n")
    end,
  })
  if not started or job <= 0 then
    vim.notify("Failed to start snapshot generator: " .. tostring(job), vim.log.levels.ERROR)
    return nil
  end

  local sent, bytes = pcall(vim.fn.chansend, job, json_string)
  pcall(vim.fn.chanclose, job, "stdin")
  local status = vim.fn.jobwait({ job })[1]
  if status < 0 then
    pcall(vim.fn.jobstop, job)
  end

  local output_path = output:gsub("\r?\n$", "")
  if status ~= 0 or not sent or bytes == 0 or output_path == "" then
    local detail = vim.trim(errors)
    if detail == "" then
      detail = "generator exited without a saved path (status " .. status .. ")"
    end
    vim.notify("Failed to generate snapshot: " .. detail, vim.log.levels.ERROR)
    return nil
  end
  if errors ~= "" then
    vim.notify(vim.trim(errors), vim.log.levels.WARN)
  end

  vim.notify("Snapshot saved to: " .. output_path, vim.log.levels.INFO)
  return output_path
end

return M
