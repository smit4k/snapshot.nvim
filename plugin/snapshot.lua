local snapshot = require("snapshot")

vim.api.nvim_create_user_command("Snapshot", function()
  print(snapshot.hello())
end, {})

vim.api.nvim_create_user_command("SnapshotBuffer", function()
  local lines = snapshot.capture_buffer()
  print(table.concat(lines, "\n"))
end, {})

vim.api.nvim_create_user_command("SnapshotVisual", function()
  local lines = snapshot.capture_visual()
  print(table.concat(lines, "\n"))
end, { range = true })

vim.api.nvim_create_user_command("SnapshotBufferJson", function()
  local bufnr = vim.api.nvim_get_current_buf()
  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
  local buffer_json = require("snapshot.json").build_snapshot_json(bufnr, lines)
  print(vim.fn.json_encode(buffer_json))
end, {})
