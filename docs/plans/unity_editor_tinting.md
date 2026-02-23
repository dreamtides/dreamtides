# Unity Editor Window Tinting

Visually differentiate multiple Unity Editor instances (main vs worktree) using
colored accent strips.

## Approach

Two pieces:

1. **Unity editor script** — Each project gets a small `[InitializeOnLoad]`
   script that prefixes the window title (e.g. `[ALPHA]` for the worktree,
   `[MAIN]` for the primary). Uses the built-in
   `EditorApplication.updateMainWindowTitle` callback.

2. **Hammerspoon script** — Draws a 1px colored strip at the top of the Unity
   window. Matches on window title to pick the color: green for main, red for
   worktree. Only visible when Unity is the frontmost app.

## Unity Editor Script

Place in `client/Assets/Editor/CustomWindowTitle.cs` per project:

```csharp
using UnityEditor;

[InitializeOnLoad]
public static class CustomWindowTitle
{
    static CustomWindowTitle()
    {
        EditorApplication.updateMainWindowTitle += OnUpdateTitle;
        EditorApplication.UpdateMainWindowTitle();
    }

    static void OnUpdateTitle(ApplicationTitleDescriptor desc)
    {
        // Use "[MAIN]" in the primary project, "[ALPHA]" in the worktree
        desc.title = "[MAIN] " + desc.title;
    }
}
```

## Hammerspoon Script

```lua
-- Unity Editor window tint: colored accent strip, color based on title
-- Load via: /opt/homebrew/bin/hs -c 'dofile("<path>")'

if _G._unityTint then
    for _, c in ipairs(_G._unityTint.canvases) do c:delete() end
    if _G._unityTint.timer then _G._unityTint.timer:stop() end
    if _G._unityTint.watcher then _G._unityTint.watcher:stop() end
    _G._unityTint = nil
end

local STRIP_HEIGHT = 1
local COLOR_MAIN  = { red = 0.1, green = 0.4, blue = 0.15, alpha = 0.9 }
local COLOR_ALPHA = { red = 0.5, green = 0.1, blue = 0.1,  alpha = 0.9 }

local strip = hs.canvas.new({ x = 0, y = 0, w = 1, h = STRIP_HEIGHT })
strip:appendElements({
    type = "rectangle",
    frame = { x = 0, y = 0, w = "100%", h = "100%" },
    fillColor = COLOR_MAIN,
    action = "fill",
})
strip:level(hs.canvas.windowLevels.floating)
strip:clickActivating(false)
strip:canvasMouseEvents(false, false)

local visible = false

local function colorForTitle(title)
    if title and title:find("ALPHA") then
        return COLOR_ALPHA
    end
    return COLOR_MAIN
end

local function showStrip(win)
    local f = win:frame()
    local color = colorForTitle(win:title())
    strip[1].fillColor = color
    strip:frame({ x = f.x, y = f.y, w = f.w, h = STRIP_HEIGHT })
    strip:show()
    visible = true
end

local function hideStrip()
    strip:hide()
    visible = false
end

local watcher = hs.application.watcher.new(function(appName, eventType, app)
    if eventType == hs.application.watcher.activated then
        if appName == "Unity" then
            local win = app:mainWindow()
            if win then showStrip(win) end
        elseif visible then
            hideStrip()
        end
    end
end)
watcher:start()

local timer = hs.timer.doEvery(0.5, function()
    local app = hs.application.frontmostApplication()
    if app and app:name() == "Unity" then
        local win = app:mainWindow()
        if win then showStrip(win) end
    end
end)

local frontApp = hs.application.frontmostApplication()
if frontApp and frontApp:name() == "Unity" then
    local win = frontApp:mainWindow()
    if win then showStrip(win) end
end

_G._unityTint = { canvases = { strip }, timer = timer, watcher = watcher }
```

## Notes

- The Hammerspoon script only handles one Unity window at a time (whichever is
  frontmost). If both are visible side-by-side, it would need two strip canvases
  and a window filter instead of an app watcher.
- The 1px strip is subtle. Bump `STRIP_HEIGHT` to 2-3 if it's hard to see.
- To load on Hammerspoon startup, add a `dofile()` call to
  `~/.hammerspoon/init.lua`.
