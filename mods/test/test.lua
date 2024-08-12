local const_ui = {}

local function init_const_ui()
    const_ui.block = clinc.widget.new("block", {borders = "all"})
end

function clinc.load()
    init_const_ui();

    clinc.register_recipe_type("test:pressing", function(key, recipe)
        local out = {}
        for i, value in ipairs(recipe) do
            out[i] = key[value] or value
        end
        return out
    end)

    local metals  = {"iron", "copper", "gold"}
    local forms = {"ingot", "sheet", "rod"}

    for _, metal in ipairs(metals) do
        for _, form in ipairs(forms) do
            local item_name = "test:" .. metal .. "_" .. form
            clinc.register_item(item_name, {
                sprite = "PLACEHOLDER"
            })
        end

        local raw = "test:raw_" .. metal
        clinc.register_item(raw, {
            sprite = "PLACEHOLDER"
        })
        
        local ingot = "test:" .. metal .. "_ingot"
        local sheet = "test:" .. metal .. "_" .. "sheet"

        clinc.register_recipe(ingot, {
            type = "smelting",
            output = ingot,
            amount = 1,
            recipe = {
                ingredient = raw,
                fuel = "default:standard_fuel",
                time = 20
            }
        })
        clinc.register_recipe(sheet, {
            type = "test:pressing",
            output = sheet,
            amount = 1,
            recipe = {
                {ingot}
            }
        })
    end

    clinc.sql_db:execute(
        [[CREATE TABLE test (
            json    JSON
        )]],
        {}
    )
    clinc.sql_db:execute(
        "INSERT INTO test (json) VALUES (?1)",
        {{test = 7}}
    )
    clinc.sql_db:execute(
        "INSERT INTO test (json) VALUES (?1)",
        {{2, 3, 4, 1}}
    )
end

local secondsBetween = 1
local timeSince = 0
function clinc.update(dt)
    timeSince = timeSince + dt
    if  timeSince >= secondsBetween then
        timeSince = timeSince - secondsBetween
    end
end

local function draw_layout()
    clinc.terminal:draw{
        direction = ":",
        {{"min", 3}, const_ui.block},
        {{"percentage", 100}, const_ui.block}
    }
end

local function draw_message(text, y, height)
    clinc.terminal:draw(
        clinc.widget.new(
            "paragraph",
            text,
            {block = const_ui.block}
        ),
        {x = 2, y = y, width = 45, height = height or 4}
    )
end

local pressin_it = false
local pressin_counter = 0
local messages = {"test", "woah", " <3 ", "yeag", "what", "huh?", "text", "gecs"}
function clinc.draw()
    for y = 0, 25, 4 do
        local text = messages[math.random(1, #messages+1)]
        draw_message(text, y)
    end
    local db_contents = clinc.sql_db:query(
        "SELECT json FROM test WHERE json IS NOT NULL",
        {}
    )

    local msg = clinc.utility.json_string_to_table(db_contents[1][1])
    local msg2 = clinc.utility.json_string_to_table(db_contents[2][1])

    draw_message(clinc.utility.table_to_string({msg, msg2}), 28, 20)

    if pressin_it then
        clinc.terminal:draw(
            clinc.widget.new(
                "paragraph",
                "pressin it",
                {block = const_ui.block}
            ),
            {x = 60, y = 4, width = 20, height = 4}
        )
    end
    
    draw_layout()
end

function clinc.update(dt)
    local key = clinc.input.key
    local is_pressed = key:is_pressed("Space")
    if is_pressed then
        clinc.sql_db:execute(
            "INSERT INTO test (num) VALUES (?1)",
            {pressin_counter}
        )
        pressin_it = true
        pressin_counter = 100
    else
        pressin_counter = math.max(-200, pressin_counter - 1)
        if pressin_counter <= 0 then 
            pressin_it = false
        end
    end

end
