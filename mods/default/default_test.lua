function clinc.load()
    clinc.register_item_tag("default:standard_fuel", {})
    
    clinc.register_item("default:stick", {
        sprite = "stick.txt"
    })
    clinc.register_tile("default:stick", {
        sprite = "stick.txt",
        drop = "default:stick"
    })

    clinc.register_item("default:long_stick", {
        sprite = "long_stick.txt"
    })
    clinc.register_recipe("default:long_stick_from_sticks", {
        type = "shaped",
        output = "default:long_stick",
        amount = 1,
        key = {
            ["/"] = "default:stick"
        },
        recipe = {
            {"/"},
            {"/"},
            {"/"},
        }
    })
end

function clinc.update(dt)
    local key = clinc.input.key;
end