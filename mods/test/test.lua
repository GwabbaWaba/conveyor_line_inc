local const_ui = {}

local function init_const_ui()
    const_ui.block = clinc.widget.new("block", {borders = "all"})
end

function clinc.load()
    init_const_ui()
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

local item_types
local item_tags
local tile_types
local tile_tags

function update_local_registry()
    item_types = clinc.sql_db:query(
        "SELECT * FROM item_types",
        {}
    );
    item_tags = clinc.sql_db:query(
        "SELECT * FROM item_tags",
        {}
    );
    tile_types = clinc.sql_db:query(
        "SELECT * FROM tile_types",
        {}
    );
    tile_tags = clinc.sql_db:query(
        "SELECT * FROM tile_tags",
        {}
    );
end

function clinc.post_load()
    update_local_registry()
end

local function get_tile_render(tile)
    local sql_db = clinc.sql_db
    local tiles = {
        air = sql_db:query([[SELECT id FROM tile_types WHERE name = "default:air"]], {})[1][1],
        grassy_dirt = sql_db:query([[SELECT id FROM tile_types WHERE name = "default:grassy_dirt"]], {})[1][1],
        dirt = sql_db:query([[SELECT id FROM tile_types WHERE name = "default:dirt"]], {})[1][1]
    }

    if tile == tiles.grassy_dirt then
        return "\x1b[38;2;0;255;0m,,\x1b[0m"
    elseif tile == tiles.dirt then
        return "\x1b[38;2;139;69;19m##\x1b[0m"
    end
    return ""
end

local screens = {
    none = function() end,

    registry = function()
        local msg = "item_types:\n"..clinc.utility.table_to_string(item_types)
        clinc.terminal:draw(
            clinc.widget.new(
                "paragraph",
                msg,
                {block = const_ui.block}
            ),
            {x = 0, y = 0, width = 50, height = clinc.terminal:window_size().height}
        )

        msg = "item_tags:\n"..clinc.utility.table_to_string(item_tags)
        clinc.terminal:draw(
            clinc.widget.new(
                "paragraph",
                msg,
                {block = const_ui.block}
            ),
            {x = 50, y = 0, width = 50, height = clinc.terminal:window_size().height}
        )

        msg = "tile_types:\n"..clinc.utility.table_to_string(tile_types)
        clinc.terminal:draw(
            clinc.widget.new(
                "paragraph",
                msg,
                {block = const_ui.block}
            ),
            {x = 100, y = 0, width = 50, height = clinc.terminal:window_size().height}
        )

        msg = "tile_tags:\n"..clinc.utility.table_to_string(tile_tags)
        clinc.terminal:draw(
            clinc.widget.new(
                "paragraph",
                msg,
                {block = const_ui.block}
            ),
            {x = 150, y = 0, width = 50, height = clinc.terminal:window_size().height}
        )
    end,

    world = function()
        local sql_db = clinc.sql_db
        local tiles = {
            air = sql_db:query([[SELECT id FROM tile_types WHERE name = "default:air"]], {})[1][1],
            grassy_dirt = sql_db:query([[SELECT id FROM tile_types WHERE name = "default:grassy_dirt"]], {})[1][1],
            dirt = sql_db:query([[SELECT id FROM tile_types WHERE name = "default:dirt"]], {})[1][1]
        }
        local world = clinc.world
        local test_chunk = world:get_chunk(0, 0, 0)

        local render_buffer = ""
        for x = 1, 32 do
            local row = test_chunk[x]
            for y = 1, 32 do
                local column = row[y]
                local top_tile = -1
                
                for z = 32, 1, -1 do
                    local tile = column[z]
                    if tile ~= tiles.air then
                        top_tile = z
                        break
                    end
                end

                if top_tile ~= -1 then
                    render_buffer = render_buffer..get_tile_render(column[top_tile])
                else
                    render_buffer = render_buffer.."  "
                end
            end
            render_buffer = render_buffer.."\n"
        end
        
        clinc.terminal:draw(
            clinc.widget.new(
                "paragraph",
                render_buffer,
                {block = const_ui.block}
            ),
            {x = 0, y = 0, width = 64, height = 32}
        )
    end
}
local screen = screens.none

function clinc.draw()
    screen()
end

local timer = 0
function clinc.update(dt)
    if screen == screens.registry then
        timer = timer + dt
        if timer >= 10 then
            update_local_registry()
        end
    end

    local key = clinc.input.key
    if key:is_pressed("r") then
        screen = screens.registry
    elseif key:is_pressed("w") then
        screen = screens.world
    elseif key:is_pressed("Space") then
        screen = screens.none
    end
end
