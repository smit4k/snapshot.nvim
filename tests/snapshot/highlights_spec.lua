local snapshot_json = require("snapshot.json")
local highlights = require("snapshot.highlights")

local function assert_spans(line, expected)
  local actual = vim.tbl_map(function(span)
    return { span.start, span["end"], span.fg }
  end, line.spans)
  assert(vim.deep_equal(actual, expected), vim.inspect({ actual = actual, expected = expected }))
end

do
  local buffer = vim.api.nvim_create_buf(false, true)
  local lines = { "before", "x café y" }
  vim.api.nvim_buf_set_lines(buffer, 0, -1, false, lines)
  vim.api.nvim_buf_call(buffer, function()
    vim.cmd([[syntax match SnapshotTestWord /café/]])
  end)
  vim.api.nvim_set_hl(0, "SnapshotTestWord", { fg = 0x123456 })

  local result = snapshot_json.build_snapshot_json(buffer, { lines[2] }, 1)
  assert_spans(result[1], { { 2, 7, "#123456" } })
  vim.api.nvim_buf_delete(buffer, { force = true })
end

do
  local buffer = vim.api.nvim_create_buf(false, true)
  local lines = { 'local value = "é"', "--[[first", "é middle", "last]]", 'local other = "é"' }
  vim.api.nvim_buf_set_lines(buffer, 0, -1, false, lines)
  vim.treesitter.highlighter.new(vim.treesitter.get_parser(buffer, "lua"), {
    queries = {
      lua = [[
        ((string) @snapshot.low (#set! priority 105))
        ((string) @snapshot.high (#set! @snapshot.high priority 110) (#offset! @snapshot.high 0 1 0 -1))
        ((string) @snapshot.later (#set! priority 100))
        (comment) @snapshot.multiline
        (identifier) @_snapshot.internal
      ]],
    },
  })
  vim.api.nvim_set_hl(0, "@snapshot.low", { fg = 0x998877 })
  vim.api.nvim_set_hl(0, "@snapshot.high", { fg = 0x555555 })
  vim.api.nvim_set_hl(0, "@snapshot.high.lua", { fg = 0xABCDEF })
  vim.api.nvim_set_hl(0, "@snapshot.later", { fg = 0xFF0000 })
  vim.api.nvim_set_hl(0, "@snapshot.multiline", { fg = 0x0A0B0C })
  vim.api.nvim_set_hl(0, "@_snapshot.internal.lua", { fg = 0xFF00FF })

  local resolve = highlights.resolve_hl
  local calls = 0
  highlights.resolve_hl = function(group)
    if group == "@snapshot.high.lua" then
      calls = calls + 1
    end
    return resolve(group)
  end
  local result = snapshot_json.build_snapshot_json(buffer, lines, 0)
  highlights.resolve_hl = resolve
  assert(calls == 1, "each highlight group should resolve only once per snapshot")
  assert_spans(result[1], { { 14, 15, "#998877" }, { 15, 17, "#abcdef" }, { 17, 18, "#998877" } })
  assert_spans(result[3], { { 0, 9, "#0a0b0c" } })
  assert_spans(snapshot_json.build_snapshot_json(buffer, { lines[3] }, 2)[1], { { 0, 9, "#0a0b0c" } })
  vim.api.nvim_buf_delete(buffer, { force = true })
end

do
  local buffer = vim.api.nvim_create_buf(false, true)
  local lines = { 'local snippet = "int value;"' }
  vim.api.nvim_buf_set_lines(buffer, 0, -1, false, lines)
  local parser = vim.treesitter.get_parser(buffer, "lua", {
    injections = {
      lua = [[
        ((string) @injection.content
          (#set! injection.language "c")
          (#set! injection.include-children)
          (#offset! @injection.content 0 1 0 -1))
      ]],
    },
  })
  vim.treesitter.highlighter.new(parser, {
    queries = {
      lua = "(string) @snapshot.embedded",
      c = "(primitive_type) @snapshot.embedded",
    },
  })
  vim.api.nvim_set_hl(0, "@snapshot.embedded.lua", { fg = 0x111111 })
  vim.api.nvim_set_hl(0, "@snapshot.embedded.c", { fg = 0x222222 })

  local result = snapshot_json.build_snapshot_json(buffer, lines, 0)
  assert_spans(result[1], { { 16, 17, "#111111" }, { 17, 20, "#222222" }, { 20, 28, "#111111" } })
  vim.api.nvim_buf_delete(buffer, { force = true })
end
