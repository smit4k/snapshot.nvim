-- main module file
local module = require("snapshot.module")

---@class Config
---@field opt string Your config option
local config = {
  opt = "Hello!",
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

return M
