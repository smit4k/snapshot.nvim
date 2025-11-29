local hl = require("snapshot.highlights")

local M = {}

M.build_snapshot_json = function(lines, extmarks)
  local json = {}

  for i, line in ipairs(lines) do
    json[i] = { text = line, spans = {} }
  end

  for _, mark in ipairs(extmarks) do
    local row = mark[1]
    local start_col = mark[2]
    local details = mark[4]

    if details and details.hl_group and json[row + 1] then
      local colors = hl.resolve_hl(details.hl_group)
      table.insert(json[row + 1].spans, {
        start = start_col,
        ["end"] = details.end_col or start_col,
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
