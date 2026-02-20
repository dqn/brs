-- Decide (song selection confirmation) skin for beatoraja Rust port.
-- Shows genre, title, artist with sliding animations before transitioning to play.

local property = {}
local filepath = {}

local header = {
	type = 6,
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

	skin.source = {
		{id = 0, path = "../system.png"},
	}
	skin.font = {
		{id = 0, path = "../VL-Gothic-Regular.ttf"}
	}

	skin.image = {
		{id = "blank", src = 0, x = 0, y = 0, w = 8, h = 8},
	}

	skin.imageset = {}
	skin.value = {}

	skin.text = {
		{id = "genre", font = 0, size = 24, ref = 13},
		{id = "title", font = 0, size = 30, ref = 12},
		{id = "artist", font = 0, size = 24, ref = 14},
	}

	skin.slider = {}

	skin.destination = {
		-- Fullscreen stage file background.
		{id = -100, dst = {
			{x = 0, y = 0, w = 1280, h = 720}
		}},
		-- Genre text slides rightward.
		{id = "genre", loop = 2000, dst = {
			{time = 0, x = 300, y = 420, w = 18, h = 18},
			{time = 2000, x = 380},
		}},
		-- Title text (static).
		{id = "title", dst = {
			{x = 340, y = 360, w = 18, h = 18}
		}},
		-- Artist text slides leftward.
		{id = "artist", loop = 2000, dst = {
			{time = 0, x = 380, y = 300, w = 18, h = 18},
			{time = 2000, x = 300},
		}},
		-- Fade-out overlay.
		{id = "blank", loop = 500, timer = 2, dst = {
			{time = 0, x = 0, y = 0, w = 1280, h = 720, a = 0},
			{time = 500, a = 255},
		}}
	}

	return skin
end

return {
	header = header,
	main = main
}
