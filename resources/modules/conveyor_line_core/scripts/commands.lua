local debug = require("resources.modules.conveyor_line_core.scripts.debug")
local commsLib = require("resources.modules.conveyor_line_core.scripts.commands_lib")

-- typed command directions aliases
local directionAliases = {["w"] = "north", ["a"] = "west", ["s"] = "south", ["d"] = "east"}
local quickEvents = commsLib.quickEvents

-- previous command inputs, tracked for previous
local prevCommands = {}


-- commands

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
    if directionAliases[inputDirection] then
        direction = directionAliases[inputDirection]
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
        Core.bufferMapRedraw()
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
    if directionAliases[inputDirection] then
        direction = directionAliases[inputDirection]
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
        Core.bufferMapRedraw()
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
    if directionAliases[inputDirection] then
        direction = directionAliases[inputDirection]
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
        Core.bufferMapRedraw()
    end
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
    ["set"] = set, ["move"] = move,  ["break"] = breakCommand, ["place"] = place,
    -- debug
    ["print"] = printCommand, ["reload"] = reload, ["time-travel"] = timeTravel
}

-- used in the event listeners to call commands and print their early return fail messages
local function runCommand(command, input)
    local output = command(input)
    if output then
        Core.print(output)
    end
    prevCommands[#prevCommands+1] = {command = command, input = input}
end

--[[
    (p | prev | previous) ((num a|b {val; ?}) ({val; ?a} a|b X)?)?
    ?a: amount of args the previous command takes
]]
local function previous(input)
  local input1 = input[1]
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
end

local key_event_functions = {quickCommand}

for i = 1, #key_event_functions do
    Core.Events.KeyEvents[#Core.Events.KeyEvents+1] = key_event_functions[i]
end

local commandEventFunctions = {commandEvent}

for i = 1, #commandEventFunctions do
    Core.Events.CommandEvents[#Core.Events.CommandEvents+1] = commandEventFunctions[i]
end