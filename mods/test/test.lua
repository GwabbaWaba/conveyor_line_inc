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

local messages = {("test "):rep(100), ("woah "):rep(100)}
local last = #messages;
local function draw_message(text, y)
    clinc.terminal:draw(
        clinc.widget.new(
            "paragraph",
            text,
            {block = const_ui.block}
        ),
        {x = 2, y = y, width = 100, height = 4}
    )
end

function clinc.draw()
    local text = messages[last]
    last = last + 1;
    if last > #messages then last = 1 end

    for y = 0, 25, 4 do
        draw_message(text, y)
    end
    
    draw_layout()
end