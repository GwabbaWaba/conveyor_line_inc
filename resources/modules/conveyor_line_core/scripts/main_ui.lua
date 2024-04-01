require("resources.modules.conveyor_line_core.scripts.init")

-- map
local mapSize = 26
local mapBoxBottom = mapSize + 3

local mapBlock = {
    type = "block",
    data = {
        title = "═|map|",
        titleAlignment = "left",
        borders = {all = true},
        borderType = "double",
    },
    rect = {
        x = 80,
        y = 2,
        width = mapSize * 2 + 2,
        height = mapSize + 2
    }
}
local mapBlockIndex = #Core.ui.UiElements+1
Core.ui.UiElements[mapBlockIndex] = mapBlock

-- coords
local function updateCoords()
    local player = Core.GameInfo.Player
    local terminal = Core.Terminal

    terminal.moveCursor(81, mapBoxBottom)
    terminal.print("═("..player.getX()..", "..player.getY()..")")
end

local postMapDrawFunctions = {updateCoords}

for i = 1, #postMapDrawFunctions do
    Core.Events.PostMapDraw[#Core.Events.PostMapDraw+1] = postMapDrawFunctions[i]
end