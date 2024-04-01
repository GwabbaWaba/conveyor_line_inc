local debug = require("resources.modules.conveyor_line_core.scripts.debug")
local commsLib = require("resources.modules.conveyor_line_core.scripts.commands_lib")

local aliases = require("resources.modules.conveyor_line_core.scripts.init").config["direction_aliases"]
local quickEvents = commsLib.quickEvents

-- previous command inputs, tracked for previous
local prevCommands = {}
local prevCommandsNextIndex = 1
local quickCommandsInLoop = 0
local ticksSinceLastQuickCommand = 0

-- commands

--[[
    alias str val
  ]]
local function alias(input)
    local alia = input.arguments[1]
    if not alia then return "arguments[1] missing" end

    local val = input.arguments[2]
    if not val then return "arguments[2] missing" end

    if val == "none" then
        aliases[alia] = nil
    else
        aliases[alia] = val
    end

    local newConfig = Core.getJSON("resources\\config\\config.json")
    newConfig["conveyor_line_core_config"]["direction_aliases"] = aliases
    Core.setJSON("resources\\config\\config.json", newConfig)
end

--[[
    help (str)?
  ]]
local function help(input)
    
end

--[[
    place str dir num
  ]]
  local function place(input)
    local player = Core.GameInfo.Player
    local map = Core.GameInfo.Map.TileMap
    local tileTypes = Core.GameInfo.Tile.Types
    local tileIdents = Core.GameInfo.Tile.Identifiers

    local inputTileName = input.arguments[1]
    local inputDirection = input.arguments[2]

    if not inputDirection or not inputTileName then 
        return "arguments missing" 
    end

    local direction = inputDirection
    if aliases[inputDirection] then
        direction = aliases[inputDirection]
    end

    local tile = tileIdents.get(inputTileName)

    if not tile then return "isn't tile type" end
    if not commsLib.isDirection(direction) then return "isn't direction" end

    local distance = 1
    local inputDistance = tonumber(input.arguments[3])
    if inputDistance then
        distance = inputDistance
    end

    local movementCapability = commsLib.findObstructableTarget(direction, distance)
    local target = movementCapability.target

    local obstructionAtPlayer = false
    if movementCapability.validity == "obstructionFound" then
        if target.x == player.getX() and target.y == player.getY() then
            obstructionAtPlayer = true
        end
    end
    local canPlace = movementCapability.validity == "noObstruction" or obstructionAtPlayer

    if canPlace and distance > 0 or not tileTypes.get(tile).solid then
        map.setFromId(target.x, target.y, tile)
        Core.GameInfo.Map.queueMapRedraw()
    end
end

--[[
    break dir num
  ]]
local function breakCommand(input)
    local player = Core.GameInfo.Player
    local map = Core.GameInfo.Map.TileMap
    local tileTypes = Core.GameInfo.Tile.Types
    local tileIdents = Core.GameInfo.Tile.Identifiers

    local inputDirection = input.arguments[1]

    if not inputDirection then return "arguments[1] missing" end

    local direction = inputDirection
    if aliases[inputDirection] then
        direction = aliases[inputDirection]
    end

    if not commsLib.isDirection(direction) then return "isn't direction" end

    local distance = 1
    local inputDistance = tonumber(input.arguments[2])
    if inputDistance then
        distance = inputDistance
    end

    local breakingCapability = commsLib.findObstructableTarget(direction, distance)
    local validity = breakingCapability.validity
    local target = breakingCapability.target

    if validity == "obstructionAtTarget" or validity == "noObstruction" then
        local airId = tileIdents.get("conveyor_line_core:air")
        if validity == "noObstruction" and map.get(target.x, target.y).type == airId then
            return
        end

        map.setFromId(target.x, target.y, airId)
        Core.GameInfo.Map.queueMapRedraw()
    end
end

--[[
    move dir num
  ]]
local function move(input)
    local player = Core.GameInfo.Player

    local inputDirection = input.arguments[1]

    if not inputDirection then return "arguments[1] missing" end

    local direction = inputDirection
    if aliases[inputDirection] then
        direction = aliases[inputDirection]
    end

    if not commsLib.isDirection(direction) then return "isn't direction" end

    local distance = 1
    local inputDistance = tonumber(input.arguments[2])
    if inputDistance then
        distance = inputDistance
    end

    local movementCapability = commsLib.findObstructableTarget(direction, distance)
    local target = movementCapability.target

    local obstructionAtPlayer = false
    if movementCapability.validity == "obstructionFound" then
        if target.x == player.getX() and target.y == player.getY() then
            obstructionAtPlayer = true
        end
    end
    local canMove = movementCapability.validity == "noObstruction" or obstructionAtPlayer

    if canMove then
        player.setPosition(target.x, target.y)
        Core.GameInfo.Map.queueMapRedraw()
    end
end

--[[
    walk dir num
  ]]
local function walk(input)
    if quickCommandsInLoop > 1 then return end
    move(input)
end

--[[
    set (key |a {key; a}) {com; b}- ({val; ?a}- |a {{{val; ?a}; b}; a})
    ?a: amount of arguments which com[i] takes

    examples:
    set w move {north, 1}
    set {w, a, s, d} {break, move} {{{north}, {north}}, {{west}, {west}}, {{south}, {south}}, {{east}, {east}}}
  ]]
local function set(input)
    local keys = input.arguments[1]
    if not keys then return "keys missing" end
    if not (type(keys) == "table") then keys = {keys} end

    local commands = input.arguments[2]
    if not commands then return "commands missing" end

    -- used to simply unbind a key/set of keys
    if commands == "none" then
        for i = 1, #input.arguments do
            quickEvents[input.arguments[i]] = nil
        end

        return
    end

    if not (type(commands) == "table") then
        commands = {commands}
    end

    local arguments = input.arguments[3]
    if not arguments then return "arguments missing" end
    
    -- try
    if not (type(arguments) == "table") then
        arguments = {[1] = arguments}
    end
    -- try harder
    if not (type(arguments[1]) == "table") then
        arguments = {[1] = arguments}
    end
    if #arguments < #commands then
        for i = 2, #commands do
            arguments[i] = arguments[1]
        end
    end
    -- try hardest
    if not (type(arguments[1][1]) == "table") then
        arguments = {[1] = arguments}
    end
    if #arguments == 1 and #keys > 1 then
        for i = 2, #commands do
            arguments[i] = arguments[1]
        end
    end

    commsLib.newKeyQuickEvent(keys, commands, arguments)
end

--[[
    print val
  ]]
local function printCommand(input)
    if input.arguments[1] then
        Core.print(input.arguments[1])
    end
end

--[[
    reload
  ]]
local function reload()
    Core.reload()
end

--[[
    time-travel num
  ]]
local function timeTravel(input)
    if not input.arguments[1] then return "amount missing" end

    Core.Tick.addTime(tonumber(input.arguments[1]))
end

-- the command names for the user end
local commandFunctions = {
    -- help
    ["help"] = help,
    -- fundamentals
    ["set"] = set, ["move"] = move, ["walk"] = walk, ["break"] = breakCommand, ["place"] = place,
    ["alias"] = alias,
    -- debug
    ["print"] = printCommand, ["reload"] = reload, ["time-travel"] = timeTravel
}

local function getWithWrapAround(num, min, max)
    if num > max then return min + num - max end
    if num < min then return max + num - min end
    return num
end

-- used in the event listeners to call commands and print their early return fail messages
local function runCommand(command, input)
    local output = command(input)
    if output then
        Core.print(output)
    end

    prevCommands[prevCommandsNextIndex] = {command = command, input = input}
    prevCommandsNextIndex = prevCommandsNextIndex + 1

    prevCommandsNextIndex = getWithWrapAround(prevCommandsNextIndex+1, 1, 10)
end

--[[
    (p | prev | previous) ((num a|b {val; ?a}) ({val; ?a} a|b X)?)?
    ?a: amount of args the previous command takes
]]
local function previous(input)
  local input1 = input.arguments[1]
  local input2 = input.arguments[2]
  
  local toRun
  if not input1 then
    toRun = prevCommands[getWithWrapAround(prevCommandsNextIndex-1, 1, 10)]
  else
    local input1AsNum = tonumber(input1)
    local argsSource
    if type(input1) == "table" then
        argsSource = input1
    else
        argsSource = input2
    end

    if input1AsNum then
        if input1AsNum > 10 then return "out of range" end
        toRun = prevCommands[11 - input1AsNum]
    end

    if argsSource then toRun.arguments = input1 end
  end

  debug.deepPrintTable(toRun)
  runCommand(toRun.command, toRun)
end
commandFunctions["p"] = previous
commandFunctions["prev"] = previous
commandFunctions["previous"] = previous

-- typed command event listener
local function commandEvent(input)
    local input = commsLib.typicalSplitInput(input)

    if input.command then
        local command = commandFunctions[input.command]
        if command then runCommand(command, input) end
    end
end

-- key command event listener
local function quickCommand(keyEvent)
    quickCommandsInLoop = quickCommandsInLoop + 1
    local keyCode = commsLib.keyCodeWithModifiers(keyEvent)
    
    local quickEvent = quickEvents[keyCode]
    if quickEvent then
        for i, quickComm in ipairs(quickEvent.commands) do
            local command = commandFunctions[quickComm]
            
            if command then 
                local inputFromQuickEvent = {
                    command = quickComm,
                    arguments = quickEvent.arguments[i]
                }
                runCommand(command, inputFromQuickEvent)
            end
        end
    end
    
    ticksSinceLastQuickCommand = 0
end

local key_event_functions = {quickCommand}

for i = 1, #key_event_functions do
    Core.Events.KeyEvents[#Core.Events.KeyEvents+1] = key_event_functions[i]
end

local commandEventFunctions = {commandEvent}

for i = 1, #commandEventFunctions do
    Core.Events.CommandEvents[#Core.Events.CommandEvents+1] = commandEventFunctions[i]
end

local function updateTickCounter()
    ticksSinceLastQuickCommand = ticksSinceLastQuickCommand + 1
    quickCommandsInLoop = 0
end

Core.Events.TickEvents[#Core.Events.TickEvents+1] = updateTickCounter