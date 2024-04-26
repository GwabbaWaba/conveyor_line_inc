local commands = require("resources.modules.conveyor_line_core.scripts.commands")


for i = 1, #commands.keyEventFunctions do
    Core.Events.KeyEvents[#Core.Events.KeyEvents+1] = commands.keyEventFunctions[i]
end

for i = 1, #commands.commandEventFunctions do
    Core.Events.CommandEvents[#Core.Events.CommandEvents+1] = commands.commandEventFunctions[i]
end
