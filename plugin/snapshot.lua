local snapshot = require("snapshot")

vim.api.nvim_create_user_command("Snapshot", function()
  print(snapshot.hello())
end, {})
