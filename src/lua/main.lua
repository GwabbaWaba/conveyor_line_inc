-- Entries defined in this table are subject to direct lookup by the rust engine
core = {
    -- the dump of all the draw calls within the current frame
    draw_buffer = {},
    -- default values are main engine events
    event_registry = {
        _load = {},
        pre_load = {},
        load = {},
        post_load = {},
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
    
    -- defined in _load, the proper definitions require functions defined by the engine after initial init, these are mainly here for lsp recognition
    -- I would make these call _G["panic!"], but that too is loaded after initial init
    register_item     = function(name, data, tags)   end,
    register_item_tag = function(name, tagged_items) end,
    register_tile     = function(name, data, tags)   end,
    register_tile_tag = function(name, tagged_tiles) end,

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
    register_recipe_type = function (name, handler)
        core.recipe_types[name] = handler
    end,
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
        CREATE TABLE assets (
            name STRING NOT NULL PRIMARY KEY,
            data STRING NOT NULL
        )
        ]],
        {}
    )

    sql_db:execute(
        [[
        CREATE TABLE item_types (
            id           SMALLINT UNSIGNED NOT NULL PRIMARY KEY,
            name         STRING            NOT NULL UNIQUE,
            default_data JSON,
            tags         JSON
        )
        ]],
        {}
    )
    sql_db:execute(
        [[
        CREATE TABLE item_tags (
            id    SMALLINT UNSIGNED NOT NULL PRIMARY KEY,
            name  STRING            NOT NULL UNIQUE,
            items JSON              NOT NULL
        )
        ]],
        {}
    )
    sql_db:execute(
        [[
        CREATE TABLE tile_types (
            id           SMALLINT UNSIGNED NOT NULL PRIMARY KEY,
            name         STRING            NOT NULL UNIQUE,
            default_data JSON,
            tags         JSON
        )
        ]],
        {}
    )
    sql_db:execute(
        [[
        CREATE TABLE tile_tags (
            id    SMALLINT UNSIGNED NOT NULL PRIMARY KEY,
            name  STRING            NOT NULL UNIQUE,
            tiles JSON              NOT NULL
        )
        ]],
        {}
    )

    -- item registry
    local greatest_item_tag_id = 0
    local greatest_item_id = 0

    clinc.register_item_tag = function(name, tagged_items)
        greatest_item_tag_id = greatest_item_tag_id + 1

        sql_db:execute(
            "INSERT INTO item_tags (id, name, items) VALUES (?1, ?2, ?3)",
            {greatest_item_tag_id, name, tagged_items}
        )

    end

    clinc.register_item = function(name, data, tags)
         greatest_item_id = greatest_item_id + 1

        sql_db:execute(
            "INSERT INTO item_types (id, name) VALUES (?1, ?2)",
            {greatest_item_id, name}
        )

        if data == nil then return end
        sql_db:execute(
            "UPDATE item_types SET default_data = ?2 WHERE id = ?1",
            {greatest_item_id, data}
        )

        if tags == nil then return end
        sql_db:execute(
            "UPDATE item_types SET tags = ?2 WHERE id = ?1",
            {greatest_item_id, tags}
        )

        for _, tag_name in ipairs(tags) do
            if next(sql_db:query("SELECT * FROM item_tags WHERE name = ?1", {tag_name})) == nil then
                clinc.register_item_tag(tag_name, {name})
            else
                local tag_table = sql_db:query(
                    "SELECT id, name, items FROM item_tags WHERE name = ?1",
                    {tag_name}
                )
                insert_if_not_present(greatest_item_id, tag_table)
                sql_db:execute(
                    "UPDATE item_tags SET items = ?2 WHERE name = ?1",
                    {tag_name, tag_table}
                )
            end

        end

    end

    -- tile registry
    local greatest_tile_tag_id = 0
    local greatest_tile_id = 0

    clinc.register_tile_tag = function(name, tagged_tiles)
        greatest_tile_tag_id = greatest_tile_tag_id + 1

        sql_db:execute(
            "INSERT INTO tile_tags (id, name, tiles) VALUES (?1, ?2, ?3)",
            {greatest_tile_tag_id, name, tagged_tiles}
        )

    end

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
            if next(sql_db:query("SELECT * FROM tile_tags WHERE name = ?1", {tag_name})) == nil then
                clinc.register_tile_tag(tag_name, {name})
            else
                local tag_table = sql_db:query(
                    "SELECT id, name, tiles FROM tile_tags WHERE name = ?1",
                    {tag_name}
                )
                insert_if_not_present(greatest_tile_id, tag_table)
                sql_db:execute(
                    "UPDATE tile_tags SET tiles = ?2 WHERE name = ?1",
                    {tag_name, tag_table}
                )
            end

        end

    end

    -- world data
    -- any tiles that can have their processing skipped when outside of load distance
    sql_db:execute(
        [[
        CREATE TABLE tile_data (
            tile_id SMALLINT UNSIGNED NOT NULL,
            x       INT               NOT NULL,
            y       SMALLINT          NOT NULL,
            z       INT               NOT NULL,
            data    JSON              NOT NULL
        )
        ]],
        {}
    )
    -- any tiles that must ALWAYS be loaded (should be nearly every machine as to keep global factories running smooth)
    sql_db:execute(
        [[
        CREATE TABLE critical_tile_data (
            tile_id SMALLINT UNSIGNED NOT NULL,
            x       INT               NOT NULL,
            y       SMALL             NOT NULL,
            z       INT               NOT NULL,
            data    JSON              NOT NULL
        )
        ]],
        {}
    )

end
