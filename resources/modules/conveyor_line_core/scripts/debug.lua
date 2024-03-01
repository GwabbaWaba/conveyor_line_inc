local function deepPrintTable(tabl, tabsForDebug)
    if tabsForDebug == nil then tabsForDebug = 0 end
    local tabs = ("    "):rep(tabsForDebug)

    for key, val in pairs(tabl) do
        if type(key) == "string" then
            key = "\""..key.."\""
        end

        if type(val) == "table" then
            Core.print(tabs.."["..tostring(key).."] = {")
            deepPrintTable(val, tabsForDebug + 1)
            Core.print(tabs.."},")
        else
            if type(val) == "string" then
                val = "\""..val.."\""
            end
            
            Core.print(tabs.."["..tostring(key).."] = "..tostring(val)..",")
        end
    end
end

return {deepPrintTable = deepPrintTable}