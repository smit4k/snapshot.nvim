# Bug Fixes and Improvements

## Fixed Issues

### 1. ✅ JSON Decode Error (E474)
**Error:** `Vim:E474: Attempt to decode a blank string`

**Cause:** The command was trying to decode an empty string when no arguments were passed.

**Fix:** Updated `plugin/snapshot.lua` to check if `opts.args` is empty before attempting JSON decode:
```lua
if opts.args and opts.args ~= "" then
  local ok, decoded = pcall(vim.fn.json_decode, opts.args)
  if ok then
    config_override = decoded
  end
end
```

### 2. ✅ Visual Selection Detection
**Issue:** Visual mode detection wasn't working properly after command execution.

**Fix:** Updated `lua/snapshot.lua` to use visual marks (`'<` and `'>`) instead of checking mode:
```lua
local start_pos = vim.fn.getpos("'<")
local end_pos = vim.fn.getpos("'>")

if start_pos[2] > 0 and end_pos[2] > 0 then
  -- Use visual selection
else
  -- Use entire buffer
end
```

### 3. ✅ Treesitter Row Offset
**Issue:** When capturing visual selections, Treesitter highlights were using wrong row indices.

**Fix:** Updated `lua/snapshot/json.lua` to accept a `start_row` parameter:
```lua
M.build_snapshot_json = function(bufnr, lines, start_row)
  start_row = start_row or 0
  for i, line in ipairs(lines) do
    local row = start_row + i - 1
    -- ...
  end
end
```

### 4. ✅ Font Loading Issue
**Issue:** Downloaded HTML instead of actual font file.

**Fix:** Updated download script to fetch FiraCode from official release:
```bash
curl -L "https://github.com/tonsky/FiraCode/releases/download/6.2/Fira_Code_v6.2.zip"
```

### 5. ✅ Font API Compatibility
**Issue:** Using `rusttype` which was incompatible with `imageproc`.

**Fix:** Switched to `ab_glyph` and `FontVec`:
```rust
use ab_glyph::{FontVec, PxScale};
let font = FontVec::try_from_vec(font_data.to_vec())?;
```

## Testing

All fixes have been verified:
- ✅ Command execution without arguments
- ✅ Visual selection capture
- ✅ Full buffer capture
- ✅ Treesitter highlight preservation
- ✅ Image generation

Run `./test.sh` or `lua verify_install.lua` to verify your installation.

## Breaking Changes

None. All changes are backward compatible.

## New Features

- Better error handling with informative messages
- Improved visual selection detection
- Automatic fallback to buffer capture if no selection
