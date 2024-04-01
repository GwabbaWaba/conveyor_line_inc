local uiElemSize = 0
setmetatable(Core.ui.UiElements, {
    __newindex = function(t, key, value)
        rawset(t, "Æ"..key, value)
        uiElemSize = uiElemSize + (value and 1 or -1)

        Core.ui.queueRedraw()
    end,
    __index = function(t, key)
        return rawget(t, "Æ"..key)
    end,

    __len = function(t)
        return rawlen(t) + uiElemSize
    end
})

local config = Core.getJSON("resources\\config\\config.json")["conveyor_line_core_config"]

return {
    config = config
}