core = {
    draw_buffer = {},
    event_registry = {
        load = {},
        draw = {},
        update = {},
    },
    items = {},
    item_tags = {},
    tiles = {},
    tile_tags = {},
    recipe_types = {
        shaped = function(key, recipe)
            local out = {}
            for i, row in ipairs(recipe) do
                out[i] = {}
                for j, value in ipairs(row) do
                    local node
                    node = key[value] or value
                    out[i][j] = node
                end
            end

            return out
        end,
        
        shapeless = function(key, recipe)
            local out = {}
            for i, value in ipairs(recipe) do
                out[i] = key[value] or value
            end
            return out
        end,

        smelting = function(_, recipe)
            return {
                ingredient = recipe.ingredient,
                fuel = recipe.fuel,
                time = recipe.time or 0
            }
        end
    },
    recipes = {},
}

local function insert_if_not_present(data, table)
    for _, value in ipairs(table) do
        if value == data then return end
    end
    table[#table+1] = data
end

clinc = {
    input = {},
    terminal = {},
    
    register_item = function(name, data)
        core.items[name] = data
        if data.tags == nil then return end

        for _, tag in ipairs(data.tags) do
            local tag_table = core.item_tags[tag]
            if tag_table then
                insert_if_not_present(name, tag_table)
            else
                core.item_tags[tag] = {name}
            end
        end
    end,

    register_item_tag = function(name, data)
        local tag_table = core.item_tags[name]
        if tag_table then
            for _, value in ipairs(data) do
                tag_table[#tag_table+1] = value
            end
        else
            core.item_tags[name] = data
        end
    end,

    register_tile = function(name, data)
        core.tiles[name] = data
        if data.tags == nil then return end

        for _, tag in ipairs(data.tags) do
            local tag_table = core.tile_tags[tag]
            if tag_table then
                insert_if_not_present(name, tag_table)
            else
                core.tile_tags[tag] = {name}
            end
        end
    end,

    register_tile_tag = function(name, data)
        local tag_table = core.tile_tags[name]
        if tag_table then
            for _, value in ipairs(data) do
                tag_table[#tag_table+1] = value
            end
        else
            core.tile_tags[name] = data
        end
    end,

    register_recipe_type = function (name, handler)
        core.recipe_types[name] = handler
    end,

    register_recipe = function(name, data)
        local final_recipe = {
            output = data.output,
            amount = data.amount or 1,
            recipe = core.recipe_types[data.type](data.key or {}, data.recipe)
        }
        
        if not core.recipes[data.type] then
            core.recipes[data.type] = {}
        end
        core.recipes[data.type][name] = final_recipe
    end
}

setmetatable(clinc, {
    __newindex = function(table, key, value)
        local event_registry = core.event_registry[key]
        if event_registry ~= nil then
            event_registry[#event_registry+1] = value
        else
            rawset(table, key, value)
        end
    end
})