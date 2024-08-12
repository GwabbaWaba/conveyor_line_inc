-- Entries defined in this table are subject to direct lookup by the rust engine
core = {
    -- the dump of all the draw calls within the current frame
    draw_buffer = {},
    -- default values are main engine events
    event_registry = {
        _load = {},
        load = {},
        world_gen = {},
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


-- Holds a standard interface for the core table which is also subject to rust meddling
clinc = {
    -- engine defined
    input = {},
    terminal = {},
    sql_db = {},
    widget = {},
    utility = {},
    world = {},
    
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

    -- TODO replace with sql storage
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
    end,

    get_tile_tag = function(name)
        return tag_table[name] or {}
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

function clinc._load()
    local sql_db = clinc.sql_db
    sql_db:execute(
        [[
        CREATE TABLE tile_types (
            id           INT    NOT NULL PRIMARY KEY,
            name         STRING NOT NULL UNIQUE,
            default_data JSON,
            tags         JSON
        );
        
        CREATE TABLE tile_tags (
            id    INT    NOT NULL PRIMARY KEY,
            name  STRING NOT NULL UNIQUE,
            tiles JSON                         --list of all tile ids tagged with this tag
        );
        ]],
        {}
    )
    local greatest_tile_id = 0
    local greatest_tag_id = 0

    clinc.register_tile = function(name, data, tags)
         greatest_tile_id = greatest_tile_id + 1

        sql_db:execute(
            "INSERT INTO tile_types (id, name) VALUES (?1, ?2)",
            {greatest_tile_id, name}
        )

        if data == nil then return end
        sql_db:execute(
            "UPDATE tile_types SET default_data = ?2 WHERE id = ?1",
            {greatest_tile_id, data}
        )

        if tags == nil then return end
        sql_db:execute(
            "UPDATE tile_types SET tags = ?2 WHERE id = ?1",
            {greatest_tile_id, tags}
        )

        for _, tag_name in ipairs(tags) do
            if sql_db:query("SELECT * FROM tile_tags WHERE name = ?1", tage_name) == {} then
                greatest_tag_id = greatest_tag_id + 1
                sql_db:execute(
                    "INSERT INTO tile_tags (id, name, tiles) VALUES (?1, ?2, ?3)",
                    {greatest_tag_id, tag_name, {name}}
                )
            else
            -- TODO this branch + testing
            end

        end
        --TODO replicate behavior of code below with sql
        --for every tag in tags, see if it exists, if it doesn't then register it, otherwise insert it into the tag
        --[[
        for _, tag in ipairs(data.tags) do
            local tag_table = core.tile_tags[tag]
            if tag_table then
                insert_if_not_present(name, tag_table)
            else
                core.tile_tags[tag] = {name}
            end
        end
        ]]

    end

end
