local hl = require("snapshot.highlights")
local M = {}

local function collect_treesitter(highlighter, bufnr, lines, start_row, resolve_colors)
  local rows = {}
  local stop_row = start_row + #lines
  highlighter.tree:parse({ start_row, stop_row })

  highlighter.tree:for_each_tree(function(tstree, tree)
    local root = tstree:root()
    local first_row, _, last_row = root:range()
    if first_row >= stop_row or last_row < start_row then
      return
    end

    local lang = tree:lang()
    local query = highlighter:get_query(lang):query()
    if not query then
      return
    end

    for id, node, metadata in query:iter_captures(root, bufnr, start_row, stop_row) do
      local name = query.captures[id]
      if name:sub(1, 1) ~= "_" then
        local colors = resolve_colors("@" .. name .. "." .. lang)
        if colors then
          local capture_metadata = metadata[id]
          local priority = tonumber(metadata.priority or capture_metadata and capture_metadata.priority) or 100
          local mark = { colors = colors, priority = priority }
          local from_row, from_col, _, to_row, to_col = unpack(vim.treesitter.get_range(node, bufnr, capture_metadata))

          for row = math.max(start_row, from_row), math.min(stop_row - 1, to_row) do
            local index = row - start_row + 1
            local line = lines[index]
            local first = row == from_row and math.max(0, from_col) or 0
            local last = row == to_row and math.min(#line, to_col) or #line
            local columns = rows[index] or {}
            rows[index] = columns
            for col = first + 1, last do
              local previous = columns[col]
              if not previous or priority >= previous.priority then
                columns[col] = mark
              end
            end
          end
        end
      end
    end
  end)

  return rows
end

local function collect_syntax(bufnr, lines, start_row, resolve_colors)
  local rows = {}
  local groups = { [0] = false }
  vim.api.nvim_buf_call(bufnr, function()
    for i, line in ipairs(lines) do
      local columns = {}
      for col = 1, #line do
        local id = vim.fn.synID(start_row + i, col, 1)
        if groups[id] == nil then
          local colors = resolve_colors(vim.fn.synIDattr(id, "name"))
          groups[id] = colors and { colors = colors } or false
        end
        columns[col] = groups[id] or nil
      end
      rows[i] = columns
    end
  end)
  return rows
end

local function build_spans(columns, length)
  local spans = {}
  if not columns then
    return spans
  end

  local start_col, colors
  for col = 1, length + 1 do
    local mark = columns[col]
    local next_colors = mark and mark.colors
    if next_colors ~= colors then
      if colors then
        spans[#spans + 1] = vim.tbl_extend("error", colors, { start = start_col - 1, ["end"] = col - 1 })
      end
      start_col, colors = col, next_colors
    end
  end
  return spans
end

M.build_snapshot_json = function(bufnr, lines, start_row)
  local json = {}
  if #lines == 0 then
    return json
  end

  bufnr = bufnr == 0 and vim.api.nvim_get_current_buf() or bufnr
  start_row = start_row or 0
  local resolved = {}
  local function resolve_colors(group)
    if resolved[group] == nil then
      local colors = hl.resolve_hl(group)
      resolved[group] = next(colors) and colors or false
    end
    return resolved[group] or nil
  end

  local highlighter = vim.treesitter.highlighter.active[bufnr]
  local rows
  if highlighter then
    rows = collect_treesitter(highlighter, bufnr, lines, start_row, resolve_colors)
  else
    rows = collect_syntax(bufnr, lines, start_row, resolve_colors)
  end

  for i, line in ipairs(lines) do
    json[i] = { text = line, spans = build_spans(rows[i], #line) }
  end
  return json
end

return M
