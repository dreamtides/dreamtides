-- unity_tint.lua
-- Draws a colored strip at the top of worktree Unity Editor windows.
-- Worktree windows are identified by a [NAME] prefix in their title,
-- set by WorktreeWindowTitle.cs.

-- Clean up previous state on reload
if _G._unityTint then
  local old = _G._unityTint
  if old.canvases then
    for _, c in pairs(old.canvases) do
      c:delete()
    end
  end
  if old.filter then
    old.filter:unsubscribeAll()
    old.filter = nil
  end
end

_G._unityTint = { canvases = {}, filter = nil }

local STRIP_HEIGHT = 5

-- Visually distinct colors (avoiding green/pure-red)
local PALETTE = {
  { red = 0.39, green = 0.58, blue = 0.93, alpha = 1.0 }, -- cornflower blue
  { red = 1.00, green = 0.75, blue = 0.00, alpha = 1.0 }, -- amber
  { red = 0.56, green = 0.37, blue = 0.90, alpha = 1.0 }, -- violet
  { red = 0.93, green = 0.36, blue = 0.48, alpha = 1.0 }, -- rose
  { red = 0.00, green = 0.70, blue = 0.68, alpha = 1.0 }, -- teal
  { red = 0.85, green = 0.65, blue = 0.13, alpha = 1.0 }, -- gold
}

local function hashName(name)
  local sum = 0
  for i = 1, #name do
    sum = sum + string.byte(name, i)
  end
  return (sum % #PALETTE) + 1
end

local function extractWorktreeName(title)
  if not title then return nil end
  return title:match("^%[([A-Z][A-Z0-9%-]*)%]")
end

local function colorForName(name)
  return PALETTE[hashName(name)]
end

local function createOrUpdateCanvas(win)
  if not win or not win:id() then return end
  local name = extractWorktreeName(win:title())
  if not name then
    -- Not a worktree window; remove any existing canvas
    local existing = _G._unityTint.canvases[win:id()]
    if existing then
      existing:delete()
      _G._unityTint.canvases[win:id()] = nil
    end
    return
  end

  local frame = win:frame()
  local c = _G._unityTint.canvases[win:id()]
  if c then
    c:frame({ x = frame.x, y = frame.y, w = frame.w, h = STRIP_HEIGHT })
  else
    c = hs.canvas.new({ x = frame.x, y = frame.y, w = frame.w, h = STRIP_HEIGHT })
    c:level(hs.canvas.windowLevels.floating)
    c:behavior(hs.canvas.windowBehaviors.transient)
    c[1] = {
      type = "rectangle",
      fillColor = colorForName(name),
      action = "fill",
    }
    c:show()
    _G._unityTint.canvases[win:id()] = c
  end
end

local function removeCanvas(win)
  if not win or not win:id() then return end
  local c = _G._unityTint.canvases[win:id()]
  if c then
    c:delete()
    _G._unityTint.canvases[win:id()] = nil
  end
end

local function hideCanvas(win)
  if not win or not win:id() then return end
  local c = _G._unityTint.canvases[win:id()]
  if c then
    c:hide()
  end
end

local function showCanvas(win)
  if not win or not win:id() then return end
  local c = _G._unityTint.canvases[win:id()]
  if c then
    c:show()
    createOrUpdateCanvas(win) -- reposition in case window moved while hidden
  end
end

local filter = hs.window.filter.new("Unity")
_G._unityTint.filter = filter

filter:subscribe(hs.window.filter.windowCreated, createOrUpdateCanvas)
filter:subscribe(hs.window.filter.windowTitleChanged, createOrUpdateCanvas)
filter:subscribe(hs.window.filter.windowMoved, createOrUpdateCanvas)
filter:subscribe(hs.window.filter.windowDestroyed, removeCanvas)
filter:subscribe(hs.window.filter.windowHidden, hideCanvas)
filter:subscribe(hs.window.filter.windowUnhidden, showCanvas)

-- Apply tint to any existing Unity windows with worktree prefixes
local existingWindows = filter:getWindows()
for _, win in ipairs(existingWindows) do
  createOrUpdateCanvas(win)
end
