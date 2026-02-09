local snapshot = require("snapshot")

-- Main snapshot command - works in both normal and visual mode
vim.api.nvim_create_user_command("Snapshot", function(opts)
  local config_override = {}
  if opts.args and opts.args ~= "" then
    local ok, decoded = pcall(vim.fn.json_decode, opts.args)
    if ok then
      config_override = decoded
    else
      vim.notify("Invalid JSON arguments: " .. opts.args, vim.log.levels.ERROR)
      return
    end
  end
  snapshot.snapshot(config_override)
end, { range = true, nargs = "?" })

-- Convenience command for visual selections
vim.api.nvim_create_user_command("SnapshotVisual", function()
  snapshot.snapshot()
end, { range = true })

-- Legacy commands for debugging
vim.api.nvim_create_user_command("SnapshotHello", function()
  print(snapshot.hello())
end, {})

vim.api.nvim_create_user_command("SnapshotBuffer", function()
  local lines = snapshot.capture_buffer()
  print(table.concat(lines, "\n"))
end, {})

vim.api.nvim_create_user_command("SnapshotBufferJson", function()
  local bufnr = vim.api.nvim_get_current_buf()
  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
  local buffer_json = require("snapshot.json").build_snapshot_json(bufnr, lines, 0)
  print(vim.fn.json_encode(buffer_json))
end, {})

vim.api.nvim_create_user_command("SnapshotVisualJson", function()
  local bufnr = vim.api.nvim_get_current_buf()
  local start_pos = vim.fn.getpos("'<")
  local end_pos = vim.fn.getpos("'>")
  local lines = vim.api.nvim_buf_get_lines(bufnr, start_pos[2] - 1, end_pos[2], false)
  local buffer_json = require("snapshot.json").build_snapshot_json(bufnr, lines, start_pos[2] - 1)
  print(vim.fn.json_encode(buffer_json))
end, { range = true })
