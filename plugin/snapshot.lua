local snapshot = require("snapshot")

vim.api.nvim_create_user_command("Snapshot", function()
  print(snapshot.hello())
end, {})

vim.api.nvim_create_user_command("SnapshotBuffer", function()
  print(snapshot.capture_buffer())
end, {})

vim.api.nvim_create_user_command("SnapshotVisual", function()
  print(snapshot.capture_visual())
end, { range = true })
