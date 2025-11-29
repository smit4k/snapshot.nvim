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
  local snapshot = require("snapshot.json")
  local lines = vim.api.nvim_buf_get_lines(0, 0, -1, false)
  local extmarks = vim.api.nvim_buf_get_extmarks(0, -1, 0, -1, { details = true })
  local buffer_json = snapshot.build_snapshot_json(lines, extmarks)
  print(vim.fn.json_encode(buffer_json))
end, {})
