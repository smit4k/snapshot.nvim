local M = {}

local function rgb_to_hex(c)
  if not c then
    return nil
  end
  return string.format("#%06x", c)
end

M.resolve_hl = function(group)
  local ok, hl = pcall(vim.api.nvim_get_hl, 0, { name = group, link = false })
  if not ok then
    return {}
  end

  return {
    fg = rgb_to_hex(hl.fg),
    bg = rgb_to_hex(hl.bg),
    bold = hl.bold or false,
    italic = hl.italic or false,
    underline = hl.underline or false,
    undercurl = hl.undercurl or false,
  }
end

return M
