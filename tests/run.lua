vim.opt.runtimepath:append(vim.fn.getcwd())

for _, spec in ipairs(vim.fn.glob("tests/snapshot/*_spec.lua", false, true)) do
  dofile(spec)
  print("Passed " .. spec)
end
