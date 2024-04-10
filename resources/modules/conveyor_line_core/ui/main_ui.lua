local init = require("resources.modules.conveyor_line_core.scripts.init")

-- positioning values
local mapSize = 26

local mapDim = {w = mapSize * 2 + 2, h = mapSize + 2}
local mapTopLeft = {x = 80, y = 2}
local mapBottomRight = {x = mapTopLeft.x + mapDim.w, y = mapTopLeft.y + mapDim.h - 1}

local leftPanelDim = {w = mapDim.w / 2, h = mapDim.h}
local leftPanelTopLeft = {x = mapTopLeft.x - leftPanelDim.w, y = mapTopLeft.y}

-- formatting values
local horizChar = "═"

-- map

local mapBorder = {
    type = "block",
    data = {
        title = horizChar.."|map|",
        titleAlignment = "left",
        borders = {all = true},
        borderType = "double",
    },
    rect = {
        x = mapTopLeft.x,
        y = mapTopLeft.y,
        width = mapDim.w,
        height = mapDim.h
    }
}
local mapBorderIndex = #Core.ui.UiElements+1
Core.ui.UiElements[mapBorderIndex] = mapBorder

local function updateCoords()
    local player = Core.GameInfo.Player
    local terminal = Core.Terminal

    terminal.moveCursor(mapTopLeft.x + 2, mapBottomRight.y)
    terminal.print("("..player.getX()..", "..player.getY()..")")
end

local postMapDrawFunctions = {updateCoords}

for i = 1, #postMapDrawFunctions do
    Core.Events.PostMapDraw[#Core.Events.PostMapDraw+1] = postMapDrawFunctions[i]
end

-- left panel
local leftPanelTitle
do
    local leftTitle = horizChar.."|i|"
    local rightTitle = "|?|*|"..horizChar
    local middleSpace = leftPanelDim.w - leftTitle:len() - rightTitle:len() + 2

    leftPanelTitle = leftTitle..("═"):rep(middleSpace)..rightTitle
end

local leftPanel = {
    type = "block",
    data = {
        title = leftPanelTitle,
        titleAlignment = "left",
        borders = {all = true},
        borderType = "double",
    },
    rect = {
        x = leftPanelTopLeft.x,
        y = leftPanelTopLeft.y,
        width = leftPanelDim.w,
        height = leftPanelDim.h
    }
}
local leftPanelIndex = #Core.ui.UiElements+1
Core.ui.UiElements[leftPanelIndex] = leftPanel