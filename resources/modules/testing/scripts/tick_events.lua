require("resources.modules.conveyor_line_core.scripts.init")

local helloBlock = {
    type = "block",
    data = {
        title = "",
        titleAlignment = "left",
        borders = {top = true},
        borderType = "double",
        style = {
            fg = {255,0,0}
        }
    },
    rect = {
        x = 0,
        y = 1,
        width = 27,
        height = 1
    }
}
local helloBlockIndex = #Core.ui.UiElements+1
Core.ui.UiElements[helloBlockIndex] = helloBlock

local tickBlock = {
    type = "block",
    data = {
        title = "0",
        titleAlignment = "left",
        borders = {none = true},
    },
    rect = {
        x = 0,
        y = 2,
        width = 1,
        height = 1
    }
}
local tickBlockIndex = #Core.ui.UiElements+1
Core.ui.UiElements[tickBlockIndex] = tickBlock

local lastPos = 0
local ticksSinceLastMove = 0

local movingRight = true;
local colors = {{127,127,0}, {0,255,0}, {0,127,127}, {0,0,255}, {127,0,127}, {255,0,0}}
local colorToUse = 1;

local function helloWorld()
    if ticksSinceLastMove < 10 then
        ticksSinceLastMove = ticksSinceLastMove + 1
        return
    end
    ticksSinceLastMove = 0

    local toPrint = "|Hello World!|"
    for _ = 0, lastPos do
        toPrint = "═"..toPrint
    end
    for _ = 10 - lastPos, 0, -1 do
        toPrint = toPrint.."═"
    end

    local tempBlock = helloBlock
    tempBlock.data.title = toPrint
    tempBlock.data.style.fg = colors[colorToUse]

    Core.ui.UiElements[helloBlockIndex] = tempBlock

    if colorToUse < #colors then
        colorToUse = colorToUse + 1
    else
        colorToUse = 1
    end

    if lastPos < 10 and movingRight then
        lastPos = lastPos + 1
    elseif lastPos > -1 then
        movingRight = false
        lastPos = lastPos - 1

        if lastPos == -1 then
            movingRight = true
        end
    end
end

local ticksElapsed = 0
local function tickCounter()
    ticksElapsed = ticksElapsed + 1

    local tempBlock = tickBlock
    tempBlock.data.title = ticksElapsed
    tempBlock.rect.width = #tostring(ticksElapsed)

    Core.ui.UiElements[tickBlockIndex] = tempBlock
end

local tickEvents = {helloWorld, tickCounter}

for i = 1, #tickEvents do
    Core.Events.TickEvents[#Core.Events.TickEvents+1] = tickEvents[i]
end