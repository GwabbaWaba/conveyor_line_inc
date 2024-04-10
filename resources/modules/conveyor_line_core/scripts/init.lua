local uiElementsSize = 0
setmetatable(Core.ui.UiElements, {
    -- replacing the key with a weird one & queueing redraw
    __newindex = function(t, key, value)
        rawset(t, "*"..key, value)
        uiElementsSize = uiElementsSize + (value and 1 or -1)

        Core.ui.queueRedraw()
    end,
    __index = function(t, key)
        return rawget(t, "*"..key)
    end,

    __len = function(t)
        return rawlen(t) + uiElementsSize
    end
})

local config = Core.getJSON("resources\\config\\config.json")["conveyor_line_core_config"]

return {
    config = config
}