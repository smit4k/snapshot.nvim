# snapshot.nvim Examples

This document shows various examples of using snapshot.nvim.

## Example 1: Simple Function

```lua
local function greet(name)
  return "Hello, " .. name .. "!"
end

print(greet("World"))
```

## Example 2: Class with Methods

```lua
local MyClass = {}
MyClass.__index = MyClass

function MyClass:new(name)
  local instance = setmetatable({}, self)
  instance.name = name
  return instance
end

function MyClass:hello()
  print("Hello from " .. self.name)
end

return MyClass
```

## Example 3: API Request

```javascript
async function fetchUser(userId) {
  const response = await fetch(`/api/users/${userId}`);
  const data = await response.json();
  return data;
}

fetchUser(123).then(user => {
  console.log(user.name);
});
```

## Usage

1. Open this file in Neovim
2. Select any code block in visual mode
3. Run `:Snapshot`
4. Check `~/snapshot.png`
