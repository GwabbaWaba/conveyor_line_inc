local commands = require("resources.modules.conveyor_line_core.scripts.commands");

local mainUi = require("resources.modules.conveyor_line_core.ui.main_ui")
local commandPromptUi = {pos = mainUi.commandPromptTopLeft, dim = mainUi.commandPromptDim}
local commandPromptTextStartPos = {x = commandPromptUi.pos.x + 2, y = commandPromptUi.pos.y + 2}
local clMaxChars = commandPromptUi.dim.w - 4;
local commandPromptClearer = (" "):rep(clMaxChars)..("\b"):rep(clMaxChars)

local CLText = {}
function CLText:new()
    local newObj = {
        relativeCursorPos = 0,
        behind = {},
        after = {},
    }
    self.__index = self
    return setmetatable(newObj, self)
end
function CLText:pushBehind(val)
    self.behind[#self.behind+1] = val
end
function CLText:popBehind()
    local temp = self.behind[#self.behind]
    self.behind[#self.behind] = nil
    return temp
end
function CLText:pushAfter(val)
    self.after[#self.after+1] = val
end
function CLText:popAfter()
    local temp = self.after[#self.after]
    self.after[#self.after] = nil
    return temp
end
function CLText:moveCursor(amount)
    self.relativeCursorPos = math.min(0, self.relativeCursorPos + amount)
end

local clText = CLText:new();

local function stringifyTable(tab)
    local out = ""
    for i =1, #tab do
        out = out..tab[i]
    end
    return out
end

local function reverse(tab)
    for i = 1, #tab//2 do
        tab[i], tab[#tab-i+1] = tab[#tab-i+1], tab[i]
    end
    return tab
end

local commandPromptSpecialActionCases = {
    ["backspace"] = function()
        clText:popBehind()
        clText:moveCursor(-1)
    end,
    ["left"] = function()
        clText:pushAfter(clText:popBehind())
        clText:moveCursor(-1)
    end,
    ["right"] = function()
        clText:pushBehind(clText:popAfter())
        clText:moveCursor(1)
    end,
    ["enter"] = function()
        local input = stringifyTable(clText.behind)..stringifyTable(reverse(clText.after))
        commands.commandEvent(input)
        clText = CLText:new()
    end,
    ["tab"] = function ()
        
    end
}




local function commandPrompt(keyEvent)
    local terminal = Core.Terminal

    if #keyEvent.code == 1 then
        clText:pushBehind(keyEvent.code)
        clText:moveCursor(1)
    else
        commandPromptSpecialActionCases[keyEvent.code]()
    end

    terminal.moveCursor(commandPromptTextStartPos.x, commandPromptTextStartPos.y)
    terminal.print(commandPromptClearer)

    local toPrint = ""

    local amountPast = math.max(0, clText.relativeCursorPos - clMaxChars)
    local behindsToPrint = math.min(clMaxChars, #clText.behind)
    for i = 1+amountPast, behindsToPrint+amountPast do
        toPrint = toPrint..clText.behind[i]
    end

    local cursor = clText.after[#clText.after] or " "
    toPrint = toPrint.."\u{001B}[38;2;0;0;0;48;2;255;255;255m"..cursor.."\u{001B}[0m"

    local aheadsToPrint = math.min(#clText.after, clMaxChars - behindsToPrint) - 1
    for i = aheadsToPrint, 1+amountPast, -1 do
        toPrint = toPrint..clText.after[i]
    end

    terminal.print(toPrint)
end

local currentTypeFrame = "commandPrompt"
local typeFrames = {
    ["commandPrompt"] = commandPrompt
}

local function typeEventHandler(keyEvent)
    if keyEvent.kind ~= "press" then return end

    typeFrames[currentTypeFrame](keyEvent)
    Core.sleep(110)
end
Core.Events.TypeEvents[#Core.Events.TypeEvents+1] = typeEventHandler