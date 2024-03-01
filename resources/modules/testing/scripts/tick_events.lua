local lastPos = 0
local ticksSinceLastMove = 0

local movingRight = true;
local colors = {"255;0;0", "127;127;0", "0;255;0", "0;127;127", "0;0;255", "127;0;127"}
local colorToUse = 1;

local function helloWorld()
    if ticksSinceLastMove < 10 then
        ticksSinceLastMove = ticksSinceLastMove + 1
        return
    end
    ticksSinceLastMove = 0

    Core.Terminal.moveCursor(lastPos, 1)
    local toPrint = "Hello World!"
    for _ = 0, lastPos do
        toPrint = " "..toPrint
    end
    for _ = 10 - lastPos, 0, -1 do
        toPrint = toPrint.." "
    end
    toPrint = "\u{001B}[38;2;0;0;0;48;2;"..colors[colorToUse].."m"..toPrint.."\u{001B}[0m"

    if colorToUse < #colors then
        colorToUse = colorToUse + 1
    else
        colorToUse = 1
    end


    Core.Terminal.print(toPrint)

    if lastPos < 10 and movingRight then
        lastPos = lastPos + 1
    elseif lastPos > 0 then
        movingRight = false
        lastPos = lastPos - 1

        if lastPos == 0 then
            movingRight = true
        end
    end
end

local ticksElapsed = 0
local function tickCounter()
    ticksElapsed = ticksElapsed + 1
    Core.Terminal.moveCursor(0, 2)
    Core.Terminal.print(ticksElapsed)
end

local tickEvents = {helloWorld, tickCounter}

for i = 1, #tickEvents do
    Core.Events.TickEvents[#Core.Events.TickEvents+1] = tickEvents[i]
end