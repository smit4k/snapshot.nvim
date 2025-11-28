local hl = require("snapshot.highlights")

local M = {}

M.build_snapshot_json = function(lines, extmarks)
  local json = {}

  for i, line in ipairs(lines) do
    json[i] = { text = line, spans = {} }
  end

  -- Map extmarks to resolved colors
  for _, mark in ipairs(extmarks) do
    local group = mark.hl_group
    if group then
      local colors = hl.resolve_hl(group)

      local row = mark.start_row + 1
      table.insert(json[row].spans, {
        start = mark.start_col,
        ["end"] = mark.end_col,
        fg = colors.fg,
        bg = colors.bg,
        bold = colors.bold,
        italic = colors.italic,
        underline = colors.underline,
        undercurl = colors.undercurl,
      })
    end
  end

  return json
end

return M
