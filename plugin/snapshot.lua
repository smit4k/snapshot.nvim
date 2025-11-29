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
