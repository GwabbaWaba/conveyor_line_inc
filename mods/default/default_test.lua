local const_ui = {}

local function init_const_ui()
    const_ui.block = clinc.widget.new("block", {borders = "none"})
end

function clinc.load()
    init_const_ui()

    clinc.register_item(
        "default:long_stick",
        {},
        {
            "default:wooden/stick"
        }
    )
    clinc.register_item(
        "default:stick",
        {},
        {
            "default:wooden/stick"
        }
    )
    clinc.register_item(
        "default:dirt",
        {},
        {
            "default:sediment"
        }
    )

    clinc.register_tile(
        "default:air",
        {},
        {
            "default:gasses"
        }
    )
    clinc.register_tile(
        "default:stick",
        {
            drop = "default:stick"
        },
        {
            "default:wooden/stick", "default:biomes/forest"
        }
    )

    clinc.register_tile(
        "default:grassy_dirt",
        {
            drop = "default:dirt"
        },
        {
            "default:sediment", "default:biomes/forest"
        }
    )
    clinc.register_tile(
        "default:dirt",
        {
            drop = "default:dirt"
        },
        {
            "default:sediment", "default:biomes/forest"
        }
    )
end

function clinc.world_gen()
    local sql_db = clinc.sql_db
    local tiles = {
        air = sql_db:query([[SELECT id FROM tile_types WHERE name = "default:air"]], {})[1][1],
        grassy_dirt = sql_db:query([[SELECT id FROM tile_types WHERE name = "default:grassy_dirt"]], {})[1][1],
        dirt = sql_db:query([[SELECT id FROM tile_types WHERE name = "default:dirt"]], {})[1][1]
    }

    local world = clinc.world
    local test_chunk = {}

    for x = 1, 32 do
        test_chunk[x] = {}
        for y = 1, 32 do
            test_chunk[x][y] = {}
            for z = 1, 32 do
                if z == 1 then
                    test_chunk[x][y][z] = tiles.dirt
                elseif z == 2 then
                    local choices = {tiles.grassy_dirt, tiles.air}
                    test_chunk[x][y][z] = choices[math.random(#choices)]
                else
                    test_chunk[x][y][z] = tiles.air
                end
            end
        end
    end

    world:set_chunk(0, 0, 0, test_chunk)
end

local timer = 0
local frames = 0
local fps = 0
local terminal = clinc.terminal

function clinc.draw()
    frames = frames + 1

    terminal:draw(
        clinc.widget.new(
            "paragraph",
            "fps:",
            {block = const_ui.block}
        ),
        {x = terminal:window_size().width-9, y = 0, width = 4, height = 1}
    )
    terminal:draw(
        clinc.widget.new(
            "paragraph",
            tostring(fps),
            {block = const_ui.block, alignment = "right"}
        ),
        {x = terminal:window_size().width, y = 0, width = 5, height = 1}
    )
end

function clinc.update(dt)
    local key = clinc.input.key;
    if key:is_pressed("End") then
        exit(0)
    end
    if key:is_pressed("Home") then
        exit(1)
    end

    latest_dt = dt
    timer = timer + dt
    if timer >= 1 then
        fps = frames
        frames = 0
        timer = 0
    end
end
