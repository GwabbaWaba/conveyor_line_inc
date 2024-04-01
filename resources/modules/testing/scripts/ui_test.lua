require("resources.modules.testing.scripts.init")

local baseBlock = {
    type = "block",
    data = {
        title = "|control group|",
        titleAlignment = "center",
        borders = {top = true},
        borderType = "double",
        style = {
            fg = {0, 255, 255}
        }
    },
    rect = {
        x = 0,
        y = 0,
        width = 20,
        height = 1
    }
}

local baseBlock2Index = #Core.ui.UiElements+1
Core.ui.UiElements[baseBlock2Index] = baseBlock
--[[
local testParagraphIndex = #Core.ui.UiElements+1
baseBlock.data.title =  "test paragraph"
Core.ui.UiElements[testParagraphIndex] = {
    type = "paragraph",
    data = {
        block = baseBlock,
        text = {
            {{"First ", "line"}},
            {{"Second"}, 
                style = {
                    modifiers = {bold = true}
                }
            },
            {{"This ", "is ", "one ", "sentence", "."}}
        },
        alignment = "left",
        trim = true
    },
    rect = {
        x = 0,
        y = 5,
        width = 10,
        height = 5
    }
}
]]
