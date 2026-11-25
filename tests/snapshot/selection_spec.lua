local snapshot = require("snapshot")
local snapshot_json = require("snapshot.json")
local build_snapshot_json = snapshot_json.build_snapshot_json
local executable = vim.fn.executable
local original = {
  jobstart = vim.fn.jobstart,
  chansend = vim.fn.chansend,
  chanclose = vim.fn.chanclose,
  jobwait = vim.fn.jobwait,
}
local saved_path = "/tmp/snapshot output.png"
local callbacks, payload
local warnings = {}

vim.fn.executable = function()
  return 1
end
vim.fn.jobstart = function(_, options)
  callbacks = options
  return 1
end
vim.fn.chansend = function(_, input)
  payload = vim.json.decode(input)
  return #input
end
vim.fn.chanclose = function()
  return 1
end
vim.fn.jobwait = function()
  callbacks.on_stdout(1, { saved_path .. "\r", "" })
  callbacks.on_stderr(1, { "Clipboard unavailable", "" })
  return { 0 }
end
local notify = vim.notify
vim.notify = function(message, level)
  if level == vim.log.levels.WARN then
    warnings[#warnings + 1] = message
  elseif level == vim.log.levels.ERROR then
    error(message)
  end
end
snapshot_json.build_snapshot_json = function(_, lines, start_row)
  local result = {}
  for i, text in ipairs(lines) do
    local spans = {}
    if start_row + i == 2 then
      spans = {
        { start = 0, ["end"] = 1, fg = "#ff0000" },
        { start = 1, ["end"] = 4, fg = "#00ff00" },
      }
    end
    result[i] = { text = text, spans = spans }
  end
  return result
end

vim.api.nvim_buf_set_lines(0, 0, -1, false, { "first", "aéX", "last" })
vim.cmd.runtime("plugin/snapshot.lua")

vim.api.nvim_win_set_cursor(0, { 2, 1 })
local keys = vim.api.nvim_replace_termcodes('v:Snapshot {"line_numbers":true}<CR>', true, false, true)
vim.api.nvim_feedkeys(keys, "nx", false)
assert(
  vim.deep_equal(payload.lines, {
    { text = "é", spans = { { start = 0, ["end"] = 2, fg = "#00ff00" } } },
  }),
  vim.inspect(payload)
)
assert(payload.config.start_line == 2, vim.inspect(payload))

vim.cmd("2,2Snapshot")
assert(#payload.lines == 1 and payload.lines[1].text == "aéX", vim.inspect(payload))
assert(payload.config.start_line == 2, vim.inspect(payload))

vim.cmd("Snapshot")
assert(#payload.lines == 3 and payload.config.start_line == 1, vim.inspect(payload))

vim.cmd('2Snapshot {"start_line":99}')
assert(payload.config.start_line == 99, vim.inspect(payload))

assert(snapshot.snapshot() == saved_path, "Snapshot must preserve spaces in its output path")
assert(warnings[#warnings] == "Clipboard unavailable", "Generator warnings must remain separate from stdout")

snapshot_json.build_snapshot_json = build_snapshot_json
vim.bo.tabstop = 4
vim.api.nvim_set_hl(0, "SnapshotTabTest", { fg = 0x123456 })
vim.cmd("syntax match SnapshotTabTest /X/")
local tabbed = { "\t\tcallback()", "é\tX" }
vim.api.nvim_buf_set_lines(0, 0, -1, false, tabbed)
vim.cmd("Snapshot")
assert(payload.lines[1].text == "        callback()", vim.inspect(payload.lines[1]))
assert(payload.lines[2].text == "é   X", vim.inspect(payload.lines[2]))
local span = payload.lines[2].spans[1]
assert(span.start == 5 and span["end"] == 6 and span.fg == "#123456", vim.inspect(span))
assert(vim.deep_equal(vim.api.nvim_buf_get_lines(0, 0, -1, false), tabbed), "Snapshot must preserve tabs in the buffer")

vim.api.nvim_win_set_cursor(0, { 2, 2 })
vim.api.nvim_feedkeys(vim.api.nvim_replace_termcodes("vl:Snapshot<CR>", true, false, true), "nx", false)
assert(#payload.lines == 1 and payload.lines[1].text == "   X", vim.inspect(payload.lines))
span = payload.lines[1].spans[1]
assert(span.start == 3 and span["end"] == 4, vim.inspect(span))

vim.bo.vartabstop = "2,3"
vim.cmd("1Snapshot")
assert(payload.lines[1].text == "     callback()", vim.inspect(payload.lines[1]))

payload = nil
local message
vim.notify = function(output)
  message = output
end
vim.cmd("Snapshot [true]")
vim.notify = notify
assert(payload == nil and message:find("Invalid JSON arguments", 1, true), "Snapshot accepts a JSON config object")

vim.fn.executable = executable
for name, fn in pairs(original) do
  vim.fn[name] = fn
end
