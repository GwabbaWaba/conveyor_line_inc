local init = require("resources.modules.conveyor_line_inc.scripts.init")

local function material_gen() 
    local metal_names = {"copper", "iron", "gold"}
    local metal_visuals = {
        copper = {
            text_color_left = {242, 86, 39},
            text_color_right = {242, 86, 39}
        },
        iron = {
            text_color_left = {171, 178, 179},
            text_color_right = {171, 178, 179}
        },
        gold = {
            text_color_left = {250, 197, 7},
            text_color_right = {250, 197, 7}
        }
    }

    local material_types = {"gear", "rod"}
    local material_visuals = {
        gear = {
            character_left = "*"
        },
        rod = {
            character_left = "/"
        }
    }

    for i = 1, #material_types do

        for j = 1, #metal_names do
            local identifier = metal_names[j].."_"..material_types[i]

            local item = {}

            local visual_data = {
                character_left = material_visuals[material_types[i]]["character_left"],
                --character_right = material_visuals[material_types[i]]["character_right"],

                text_color_left = metal_visuals[metal_names[j]]["text_color_left"],
                text_color_right = metal_visuals[metal_names[j]]["text_color_right"],
                --back_color_left = metal_visuals[metal_names[j]]["back_color_left"],
                --back_color_right = metal_visuals[metal_names[j]]["back_color_right"]
            }

            Core.InitializationInfo.GameData[init.conveyorLineCoresIndex][identifier] = {
                item = item,
                visual_data = visual_data
            }

        end
    end
end

local functions = {material_gen}

for i = 1, #functions do
    Core.Events.PostDeserializationEvents[#Core.Events.PostDeserializationEvents+1] = functions[i]
end