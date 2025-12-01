local hl = require("snapshot.highlights")
local M = {}

local function get_treesitter_highlights(bufnr, row, line)
  local highlights = {}
  local len = #line

  if len == 0 then
    return highlights
  end

  local ok, parser = pcall(vim.treesitter.get_parser, bufnr)
  if not ok or not parser then
    return highlights
  end

  for col = 0, len - 1 do
    local captures = vim.treesitter.get_captures_at_pos(bufnr, row, col)
    if #captures > 0 then
      -- Use the last (highest priority) capture
      local capture = captures[#captures]
      local hl_group = "@" .. capture.capture
      table.insert(highlights, {
        col = col,
        hl_group = hl_group,
      })
    end
  end

  return highlights
end

local function merge_highlights_to_spans(highlights, line)
  local spans = {}
  if #highlights == 0 then
    return spans
  end

  local current = highlights[1]
  local start_col = current.col

  for i = 2, #highlights do
    local h = highlights[i]
    if h.hl_group ~= current.hl_group or h.col ~= highlights[i - 1].col + 1 then
      local colors = hl.resolve_hl(current.hl_group)
      table.insert(spans, {
        start = start_col,
        ["end"] = highlights[i - 1].col + 1,
        fg = colors.fg,
        bg = colors.bg,
        bold = colors.bold,
        italic = colors.italic,
        underline = colors.underline,
        undercurl = colors.undercurl,
      })
      current = h
      start_col = h.col
    end
  end

  -- Add final span
  local colors = hl.resolve_hl(current.hl_group)
  table.insert(spans, {
    start = start_col,
    ["end"] = highlights[#highlights].col + 1,
    fg = colors.fg,
    bg = colors.bg,
    bold = colors.bold,
    italic = colors.italic,
    underline = colors.underline,
    undercurl = colors.undercurl,
  })

  return spans
end

M.build_snapshot_json = function(bufnr, lines)
  local json = {}

  for i, line in ipairs(lines) do
    local row = i - 1
    local highlights = get_treesitter_highlights(bufnr, row, line)
    local spans = merge_highlights_to_spans(highlights, line)

    json[i] = { text = line, spans = spans }
  end

  return json
end

return M
