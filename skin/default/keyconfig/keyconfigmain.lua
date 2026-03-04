-- Key configuration skin for beatoraja Rust port.
-- Minimal placeholder skin (key binding is handled by the engine UI).

local property = {}
local filepath = {}

local header = {
	type = 8,
	name = "beatoraja default (lua)",
	w = 1280,
	h = 720,
	scene = 3000,
	input = 500,
	fadeout = 500,
	property = property,
	filepath = filepath
}

local function main()
	local skin = {}
	for k, v in pairs(header) do
		skin[k] = v
	end

	skin.source = {}
	skin.font = {}
	skin.image = {}
	skin.imageset = {}
	skin.value = {}
	skin.text = {}
	skin.slider = {}
	skin.destination = {}

	return skin
end

return {
	header = header,
	main = main
}
