-- unity_tint.lua
-- Draws a colored strip at the top of worktree Unity Editor windows.
-- Worktree windows are identified by a [NAME] prefix in their title,
-- set by WorktreeWindowTitle.cs. Strips are only visible when Unity is
-- the frontmost application.

-- Clean up previous state on reload
if _G._unityTint then
  local old = _G._unityTint
  if old.canvases then
    for _, c in pairs(old.canvases) do
      c:delete()
    end
  end
  if old.timer then old.timer:stop() end
  if old.watcher then old.watcher:stop() end
end

_G._unityTint = { canvases = {}, timer = nil, watcher = nil }

local STRIP_HEIGHT = 1

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

local function showStrips()
  local app = hs.application.frontmostApplication()
  if not app or app:name() ~= "Unity" then return end

  local seen = {}
  for _, win in ipairs(app:allWindows()) do
    local wid = win:id()
    if wid then
      local name = extractWorktreeName(win:title())
      if name then
        seen[wid] = true
        local f = win:frame()
        local c = _G._unityTint.canvases[wid]
        if c then
          c:frame({ x = f.x, y = f.y, w = f.w, h = STRIP_HEIGHT })
          c[1].fillColor = colorForName(name)
          c:show()
        else
          c = hs.canvas.new({ x = f.x, y = f.y, w = f.w, h = STRIP_HEIGHT })
          c:level(hs.canvas.windowLevels.floating)
          c:clickActivating(false)
          c:canvasMouseEvents(false, false)
          c[1] = {
            type = "rectangle",
            fillColor = colorForName(name),
            action = "fill",
          }
          c:show()
          _G._unityTint.canvases[wid] = c
        end
      end
    end
  end

  -- Remove canvases for windows that no longer exist or lost their prefix
  for wid, c in pairs(_G._unityTint.canvases) do
    if not seen[wid] then
      c:delete()
      _G._unityTint.canvases[wid] = nil
    end
  end
end

local function hideAllStrips()
  for _, c in pairs(_G._unityTint.canvases) do
    c:hide()
  end
end

local watcher = hs.application.watcher.new(function(appName, eventType)
  if eventType == hs.application.watcher.activated then
    if appName == "Unity" then
      showStrips()
    else
      hideAllStrips()
    end
  end
end)
watcher:start()
_G._unityTint.watcher = watcher

-- Poll to track window moves/resizes while Unity is frontmost
local timer = hs.timer.doEvery(0.5, function()
  local app = hs.application.frontmostApplication()
  if app and app:name() == "Unity" then
    showStrips()
  end
end)
_G._unityTint.timer = timer

-- Apply immediately if Unity is already frontmost
local frontApp = hs.application.frontmostApplication()
if frontApp and frontApp:name() == "Unity" then
  showStrips()
end
