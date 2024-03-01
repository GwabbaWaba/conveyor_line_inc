-- quick event keybinds
local quickEvents = {
    ["j"] = {
        ["arguments"] = {
            [1] = {
                [1] = "south",
            },
        },
        ["commands"] = {
            [1] = "move",
        },
    },
    ["k"] = {
        ["arguments"] = {
            [1] = {
                [1] = "north",
            },
        },
        ["commands"] = {
            [1] = "move",
        },
    },
    ["h"] = {
        ["arguments"] = {
            [1] = {
                [1] = "west",
            },
        },
        ["commands"] = {
            [1] = "move",
        },
    },
    ["D"] = {
        ["arguments"] = {
            [1] = {
                [1] = "east",
            },
        },
        ["commands"] = {
            [1] = "break",
        },
    },
    ["S"] = {
        ["arguments"] = {
            [1] = {
                [1] = "south",
            },
        },
        ["commands"] = {
            [1] = "break",
        },
    },
    ["W"] = {
        ["arguments"] = {
            [1] = {
                [1] = "north",
            },
        },
        ["commands"] = {
            [1] = "break",
        },
    },
    ["l"] = {
        ["arguments"] = {
            [1] = {
                [1] = "east",
            },
        },
        ["commands"] = {
            [1] = "move",
        },
    },
    ["A"] = {
        ["arguments"] = {
            [1] = {
                [1] = "west",
            },
        },
        ["commands"] = {
            [1] = "break",
        },
    },
    ["H"] = {
        ["arguments"] = {
            [1] = {
                [1] = "west",
            },
        },
        ["commands"] = {
            [1] = "break",
        },
    },
    ["s"] = {
        ["arguments"] = {
            [1] = {
                [1] = "south",
            },
        },
        ["commands"] = {
            [1] = "move",
        },
    },
    ["J"] = {
        ["arguments"] = {
            [1] = {
                [1] = "south",
            },
        },
        ["commands"] = {
            [1] = "break",
        },
    },
    ["a"] = {
        ["arguments"] = {
            [1] = {
                [1] = "west",
            },
        },
        ["commands"] = {
            [1] = "move",
        },
    },
    ["L"] = {
        ["arguments"] = {
            [1] = {
                [1] = "east",
            },
        },
        ["commands"] = {
            [1] = "break",
        },
    },
    ["w"] = {
        ["arguments"] = {
            [1] = {
                [1] = "north",
            },
        },
        ["commands"] = {
            [1] = "move",
        },
    },
    ["d"] = {
        ["arguments"] = {
            [1] = {
                [1] = "east",
            },
        },
        ["commands"] = {
            [1] = "move",
        },
    },
    ["K"] = {
        ["arguments"] = {
            [1] = {
                [1] = "north",
            },
        },
        ["commands"] = {
            [1] = "break",
        },
    },
    ["."] = {
        ["arguments"] = {
            [1] = {
                [1] = "north",
                [2] = "0",
            },
        },
        ["commands"] = {
            [1] = "break",
        },
    },
    ["F12"] = {
        ["arguments"] = {},
        ["commands"] = {
            [1] = "reload",
        },
    },
    ["F10"] = {
        ["arguments"] = {
            [1] = {
                [1] = "20",
            },
        },
        ["commands"] = {
            [1] = "time-travel",
        },
    }
}

--[[
    all functions until typicalSplitInput are designed for that function
  ]]
local function findNonTableZones(positionsOfTables, str)
    local zonesOutsideTables = {}

    local endPos = positionsOfTables[1];
    if not (endPos == nil) then
        endPos = endPos.startPos
    end

    zonesOutsideTables[1] = {
        zone = str:sub(1, endPos),
        insertPos = 1
    }

    for i = 1, #positionsOfTables do
        local startPos = positionsOfTables[i].endPos + 1

        local endPos = positionsOfTables[i+1];
        if not (endPos == nil) then
            endPos = endPos.startPos
            if startPos > endPos then
                startPos = 1
            end
        end

        zonesOutsideTables[#zonesOutsideTables+1] = {
            zone = str:sub(startPos, endPos),
            insertPos = i
        }
    end

    return zonesOutsideTables
end

local function insertValuesFromZones(zonesOutsideTables, tabl)
    local jump = 0
    for i = 1, #zonesOutsideTables do
        local insertPos = zonesOutsideTables[i].insertPos + jump
        
        for argument in zonesOutsideTables[i].zone:gmatch("([^%s,{}]+)") do
            if insertPos > #tabl + 1 then
                insertPos = #tabl + 1
            end

            table.insert(tabl, insertPos, argument)
            insertPos = insertPos + 1
        end

        jump = insertPos
    end

    return tabl
end

local function processInputTable(tableContents)
    local properArguments = {}
    local positionsOfTables = {}

    for startPos, tableArgument, endPos in tableContents:gmatch("()(%b{})()") do
        positionsOfTables[#positionsOfTables+1] = {startPos = startPos, endPos = endPos}
        
        table.insert(properArguments, processInputTable(tableArgument:sub(2, -2)))
    end

    local zonesOutsideTables = findNonTableZones(positionsOfTables, tableContents)
    properArguments = insertValuesFromZones(zonesOutsideTables, properArguments)

    return properArguments
end

local function typicalSplitInput(input)
    local splitInput = {
        command = nil,
        arguments = {}
    }
    local command, endOfCommand = input:match("^([^%s,]*) ?()")
    splitInput.command = command
    if not endOfCommand then return splitInput end
    
    local remaining = input:sub(endOfCommand)

    local arguments = {}
    local positionsOfTables = {}

    for startPos, argument, endPos in remaining:gmatch("()(%b{})()") do
        local table = processInputTable(argument:sub(2, -2))
        
        arguments[#arguments+1] = table
        positionsOfTables[#positionsOfTables+1] = {startPos = startPos, endPos = endPos}
    end

    local zonesOutsideTables = findNonTableZones(positionsOfTables, remaining)
    splitInput.arguments = insertValuesFromZones(zonesOutsideTables, arguments)

    return splitInput
end

-- returns if dubiousDir is any of the follwing, doesn't return which one: "north", "east", "south", or "west"
local function isDirection(dubiousDir)
    return dubiousDir == "north" or dubiousDir == "east" or dubiousDir == "south" or dubiousDir == "west"
end

--[[
    used to find a path for a command
    examples from conveyor_line_core's command set:
    used in move to determine if an obstacle is in the way from player to target
    used in break to determine if there is something to break at target, and also if the player has direct line of sight
  ]]
local function findObstructableTarget(direction, distance)
    local gameInfo = Core.GameInfo
    local playerPos = {x = gameInfo.Player.getX(), y = gameInfo.Player.getY()}

    local horizontal = true;
    local target = nil
    local iter = nil;

    if direction == "north" then
        if playerPos.y < distance then return {validity = "outOfBounds"} end
        target = {x = playerPos.x, y = playerPos.y - distance}

        iter = {s = target.y, e = playerPos.y, m = 1}
        horizontal = false;
    elseif direction == "east" then
        target = {x = playerPos.x + distance, y = playerPos.y}

        iter = {s = target.x, e = playerPos.x, m = -1}
    elseif direction == "south" then
        target = {x = playerPos.x, y = playerPos.y + distance}

        iter = {s = target.y, e = playerPos.y, m = -1}
        horizontal = false;
    elseif direction == "west" then
        if playerPos.x < distance then return {validity = "outOfBounds"} end
        target = {x = playerPos.x - distance, y = playerPos.y}

        iter = {s = target.x, e = playerPos.x, m = 1}
    else
        return {validity = "invalidDirection"}
    end

    if target.x > gameInfo.Map.width or target.y > gameInfo.Map.height then return {validity = "outOfBounds"} end

    local i = 1;
    for coord = iter.s, iter.e, iter.m do
        local targettedTile = nil
        if horizontal then
            targettedTile = gameInfo.Map.TileMap.get(coord, playerPos.y)
        else
            targettedTile = gameInfo.Map.TileMap.get(playerPos.x, coord)
        end

        if gameInfo.Tile.Types.get(targettedTile.type).solid then
            local obstructionPosition = nil
            if horizontal then
                obstructionPosition = {x = coord, y = playerPos.y}
            else
                obstructionPosition = {x = playerPos.x, y = coord}
            end

            if i == distance then
                return {validity = "obstructionAtTarget", obstructionPosition = obstructionPosition, target = target}
            end
            return {validity = "obstructionFound", obstructionPosition = obstructionPosition, target = target}
        end
        i = i + 1;
    end

    return {validity = "noObstruction", target = target}
end

--[[
    adds a quick event to quickEvents using a tables in the format of:
    keys={keys; a} commands={commands; b} arguments={{{args for commands}; b}; a}
  ]]
local function newKeyQuickEvent(keys, commands, arguments)
    for i = 1, #keys do

        local newQuickEvent = {
            commands = commands,
            arguments = arguments[i]
        }
        quickEvents[keys[i]] = newQuickEvent
    end
end

--[[
    formats a key code with its modifiers, ignoring shift on single characters
    example:
    (non-relevent fields ommitted)
    keyEvent={modifiers = {control = true, shift = true}, code = "L"} ->  "c+L"
  ]]
local function keyCodeWithModifiers(keyEvent)
    local control = ""
    local alt = ""
    local shift = ""
    if keyEvent.modifiers.control then control = "c+" end
    if keyEvent.modifiers.alt then alt = "a+" end
    if keyEvent.modifiers.shift and keyEvent.code:len() > 1 then control = "s+" end

    return control..alt..shift..keyEvent.code
end

return {
    quickEvents = quickEvents,
    findNonTableZones = findNonTableZones,
    insertValuesFromZones = insertValuesFromZones,
    processInputTable = processInputTable,
    typicalSplitInput = typicalSplitInput,
    isDirection = isDirection,
    findObstructableTarget = findObstructableTarget,
    newKeyQuickEvent = newKeyQuickEvent,
    keyCodeWithModifiers = keyCodeWithModifiers
}