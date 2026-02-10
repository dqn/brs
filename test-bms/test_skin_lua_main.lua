local main_state = {}

main_state.header = {
    type = 7,
    name = "Test Lua Skin",
    w = 1920,
    h = 1080,
    property = {
        {
            name = "Effect",
            item = {
                {name = "ON", op = 800},
                {name = "OFF", op = 801}
            }
        }
    },
    filepath = {
        {name = "Wallpaper", path = "wall/*.png", def = "wall.png"}
    },
    offset = {
        {name = "Score Pos", id = 10, x = true, y = true, w = false, h = false, r = false, a = false},
        {name = "Judge Pos", id = 11, x = true, y = true, w = false, h = false, r = false, a = false}
    }
}

function main_state.main()
    local skin = {}
    for k, v in pairs(main_state.header) do
        skin[k] = v
    end
    skin.source = {{id = 0, path = "test.png"}}
    skin.font = {{id = 0, path = "test.ttf"}}
    skin.image = {
        {id = "bg", src = 0, x = 0, y = 0, w = 1920, h = 1080}
    }
    skin.text = {
        {id = "title", font = 0, size = 24, ref = 12}
    }
    skin.scene = 5000
    skin.input = 500
    skin.fadeout = 600

    local dst = {}
    dst[#dst + 1] = {
        id = "bg",
        dst = {{time = 0, x = 0, y = 0, w = 1920, h = 1080, a = 255}}
    }
    dst[#dst + 1] = {
        id = "title",
        dst = {{time = 0, x = 10, y = 10, w = 800, h = 30, a = 255}}
    }

    if skin_config then
        local opts = skin_config.enabled_options or {}
        for _, v in ipairs(opts) do
            if v == 800 then
                skin.image[#skin.image + 1] = {
                    id = "effect", src = 0, x = 0, y = 0, w = 100, h = 100
                }
                dst[#dst + 1] = {
                    id = "effect",
                    dst = {{time = 0, x = 500, y = 500, w = 100, h = 100, a = 200}}
                }
            end
        end
    end
    skin.destination = dst
    return skin
end

return main_state
