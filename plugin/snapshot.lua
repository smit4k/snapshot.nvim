local snapshot = require("snapshot")

vim.api.nvim_create_user_command("snapshot", function()
  print(snapshot.hello())
end, {})
