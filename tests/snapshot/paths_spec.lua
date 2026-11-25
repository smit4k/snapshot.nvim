local repo = vim.fn.fnamemodify(debug.getinfo(1, "S").source:sub(2), ":p:h:h:h")
local root = vim.fn.tempname() .. " custom checkout"
local module_path = root .. "/lua/snapshot.lua"
local executable = vim.fn.executable
local notify = vim.notify

local ok, err = pcall(function()
  vim.fn.mkdir(root .. "/lua", "p")
  assert(vim.fn.writefile(vim.fn.readfile(repo .. "/lua/snapshot.lua", "b"), module_path, "b") == 0)

  local snapshot = assert(loadfile(module_path))()
  local generator_bin = vim.fn.has("win32") == 1 and "snapshot-generator.exe" or "snapshot-generator"
  local expected = vim.fn.fnamemodify(root, ":p"):gsub("[/\\]$", "") .. "/generator/target/release/" .. generator_bin
  local lookups = 0
  vim.fn.executable = function(path)
    assert(path == expected, "Unexpected generator path: " .. path)
    lookups = lookups + 1
    return 0
  end
  vim.notify = function() end
  snapshot.install = function() end

  snapshot.setup()
  assert(lookups > 0, "Setup did not look for the generator")
  lookups = 0
  snapshot.snapshot()
  assert(lookups > 0, "Capture did not look for the generator")
  lookups = 0
  snapshot.debug()
  assert(lookups > 0, "Debug did not look for the generator")
end)

vim.fn.executable = executable
vim.notify = notify
vim.fn.delete(root, "rf")
assert(ok, err)
