local snapshot = require("snapshot")

-- Main snapshot command - works in both normal and visual mode
vim.api.nvim_create_user_command("Snapshot", function(opts)
  local config_override = {}
  -- Check if args exists and is not empty or whitespace
  if opts.args and opts.args:match("%S") then
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

-- Debug command to check paths and installation
vim.api.nvim_create_user_command("SnapshotDebug", function(opts)
  print("=== Snapshot Debug Info ===")
  print("opts.args type: " .. type(opts.args))
  print("opts.args value: " .. vim.inspect(opts.args))
  print("")

  -- Check runtime paths
  print("Checking runtime paths for snapshot.nvim...")
  local rtp = vim.api.nvim_list_runtime_paths()
  local found_plugin = false
  for _, path in ipairs(rtp) do
    if path:match("snapshot%.nvim") or path:match("snapshot$") then
      found_plugin = true
      print("✓ Plugin path: " .. path)

      local gen_path = path .. "/generator/target/release/snapshot-generator"
      if vim.fn.executable(gen_path) == 1 then
        print("✓ Generator found: " .. gen_path)
        local size = vim.fn.getfsize(gen_path)
        print("  Size: " .. string.format("%.1f MB", size / 1024 / 1024))
      else
        print("✗ Generator NOT found: " .. gen_path)
        print("  Run: cd " .. path .. "/generator && cargo build --release")
      end
    end
  end

  if not found_plugin then
    print("✗ snapshot.nvim not found in runtimepath")
  end

  print("")
  print("Module location: " .. debug.getinfo(1, "S").source)
end, { nargs = "?" })
