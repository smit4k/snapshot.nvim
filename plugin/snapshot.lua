local ok, snapshot = pcall(require, "snapshot")
if not ok then
  vim.notify("snapshot.nvim: failed to load module: " .. tostring(snapshot), vim.log.levels.ERROR)
  return
end

-- Ex resolves both Visual marks and numeric ranges to line numbers before invoking the command.
local visual_command
vim.api.nvim_create_autocmd("CmdlineLeave", {
  group = vim.api.nvim_create_augroup("SnapshotCommand", { clear = true }),
  pattern = ":",
  callback = function()
    local command = vim.fn.getcmdline()
    if not vim.v.event.abort and command:match("^%s*'<,'>%s*Snapshot%f[%W]") then
      visual_command = command
      vim.schedule(function()
        if visual_command == command then
          visual_command = nil
        end
      end)
    end
  end,
})

vim.api.nvim_create_user_command("Snapshot", function(opts)
  local visual = visual_command ~= nil
  visual_command = nil
  local config_override = {}
  -- Check if args exists and is not empty or whitespace
  if opts.args and opts.args:match("%S") then
    local decode_ok, decoded = pcall(vim.fn.json_decode, opts.args)
    if decode_ok and opts.args:match("^%s*{") then
      config_override = decoded
    else
      vim.notify("Invalid JSON arguments: " .. opts.args, vim.log.levels.ERROR)
      return
    end
  end
  local range = opts.range > 0 and { line1 = math.max(opts.line1, 1), line2 = math.max(opts.line2, 1), visual = visual }
    or nil
  snapshot.snapshot(config_override, range)
end, { range = true, nargs = "?" })

vim.api.nvim_create_user_command("SnapshotDebug", snapshot.debug, { nargs = "?" })
