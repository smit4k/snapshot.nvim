local root = vim.fn.tempname() .. "/snapshot.nvim"
local release_dir = root .. "/generator/target/release/"
local is_windows = vim.fn.has("win32") == 1 or vim.fn.has("win64") == 1
local executable_name = is_windows and "snapshot-generator.exe" or "snapshot-generator"
local font = release_dir .. "JetBrainsMono-Regular.ttf"
local version = release_dir .. ".version"

vim.fn.mkdir(root .. "/lua", "p")
vim.fn.writefile(vim.fn.readfile("lua/snapshot.lua"), root .. "/lua/snapshot.lua")
local snapshot = dofile(root .. "/lua/snapshot.lua")
local executable = vim.fn.executable
local system = vim.fn.system
vim.fn.executable = function(path)
  return vim.fn.filereadable(path)
end

local fail_font = true
local cargo_args
local cargo_calls = 0
vim.fn.system = function(args)
  if args[1] == "cargo" then
    cargo_args = args
    cargo_calls = cargo_calls + 1
    vim.fn.writefile({ "generator" }, release_dir .. executable_name)
  elseif args[1] == "curl" then
    if args[4]:find("/smit4k/", 1, true) or fail_font then
      error("Simulated download failure")
    end
  elseif args[1] == "unzip" or args[1] == "powershell" then
    vim.fn.writefile({ "font" }, font)
  end
  return ""
end

local ok, failure = pcall(function()
  assert(not snapshot.install())
  assert(vim.fn.filereadable(version) == 0, "A partial install must not write a version")
  assert(cargo_args[5] == root .. "/generator/Cargo.toml", vim.inspect(cargo_args))
  assert(cargo_args[7] == root .. "/generator/target", vim.inspect(cargo_args))
  fail_font = false
  snapshot.setup()
  assert(vim.fn.filereadable(version) == 1, "Setup must retry a partial install")
  assert(vim.fn.filereadable(font) == 1)
  assert(cargo_calls == 2)
  snapshot.setup()
  assert(cargo_calls == 2, "A matching installed version should be reused")
  vim.fn.delete(font)
  snapshot.setup()
  assert(cargo_calls == 3 and vim.fn.filereadable(font) == 1, "Setup must repair a missing font")
end)

vim.fn.executable = executable
vim.fn.system = system
vim.fn.delete(vim.fn.fnamemodify(root, ":h"), "rf")
assert(ok, failure)
