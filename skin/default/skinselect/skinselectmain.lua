-- Skin selection screen for beatoraja Rust port.
-- Allows browsing skin types, previewing skins, and adjusting custom properties.

local property = {}
local filepath = {}

local header = {
	type = 9,
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
		{id = 0, path = "../skinselect.png"},
	}
	skin.font = {
		{id = 0, path = "../VL-Gothic-Regular.ttf"}
	}

	-- Skin type labels: each type has an off (unselected) and on (selected) state.
	-- The type ids map to the engine's skin type constants.
	local skin_types = {5,6,7,15,10,8,9,11,0,12,2,1,13,3,4,14,16,18,17}
	skin.image = {
		{id = "preview-bg", src = 0, x = 0, y = 664, w = 640, h = 360},
		{id = "arrow-l", src = 0, x = 989, y = 0, w = 12, h = 12},
		{id = "arrow-r", src = 0, x = 1001, y = 0, w = 12, h = 12},
		{id = "arrow-l-active", src = 0, x = 989, y = 12, w = 12, h = 12},
		{id = "arrow-r-active", src = 0, x = 1001, y = 12, w = 12, h = 12},
		{id = "scroll-bg", src = 0, x = 1014, y = 0, w = 10, h = 251},

		-- Skin preview click area (invisible button).
		{id = "button-skin", src = 0, x = 640, y = 0, w = 0, h = 0, act = 190, click = 2},
	}

	-- Custom property change buttons (6 slots).
	for slot = 1, 6 do
		table.insert(skin.image, {
			id = "button-custom-" .. slot,
			src = 0, x = 640, y = 10, w = 120, h = 48,
			act = 219 + slot, click = 2
		})
	end

	-- Skin type label images (off / on states).
	for idx, tid in ipairs(skin_types) do
		local row_y = (idx - 1) * 30
		table.insert(skin.image, {id = "type-off-" .. tid, src = 0, x = 0, y = row_y, w = 300, h = 30})
		table.insert(skin.image, {id = "type-on-" .. tid, src = 0, x = 300, y = row_y, w = 300, h = 30})
	end

	-- Imageset toggles for skin types.
	skin.imageset = {}
	local type_act_map = {
		[0] = 170, [1] = 171, [2] = 172, [3] = 173, [4] = 174,
		[5] = 175, [6] = 176, [7] = 177, [8] = 178, [9] = 179,
		[10] = 180, [11] = 181, [12] = 182, [13] = 183, [14] = 184,
		[15] = 185, [16] = 386, [17] = 387, [18] = 388,
	}
	for _, tid in ipairs(skin_types) do
		local act_id = type_act_map[tid]
		table.insert(skin.imageset, {
			id = "type-" .. tid,
			images = {"type-off-" .. tid, "type-on-" .. tid},
			act = act_id, ref = act_id
		})
	end

	skin.value = {}

	-- Text labels for skin name and custom property labels/values.
	skin.text = {
		{id = "skin-name", font = 0, size = 24, align = 1, ref = 50},
	}
	for slot = 1, 6 do
		table.insert(skin.text, {id = "custom-label-" .. slot, font = 0, size = 24, align = 2, ref = 99 + slot})
		table.insert(skin.text, {id = "custom-value-" .. slot, font = 0, size = 24, align = 1, ref = 109 + slot})
	end

	skin.slider = {
		{id = "scroll-fg", src = 0, x = 1007, y = 252, w = 17, h = 24, angle = 2, range = 232, type = 7},
	}

	skin.destination = {
		-- Skin preview area (click target).
		{id = "button-skin", dst = {
			{x = 450, y = 350, w = 680, h = 360},
		}},
		-- Preview background.
		{id = "preview-bg", dst = {
			{x = 470, y = 350, w = 640, h = 360},
		}},
		-- Skin name (centered above preview).
		{id = "skin-name", dst = {
			{x = 790, y = 310, w = 640, h = 24},
		}},
		-- Navigation arrows for skin browsing.
		{id = "arrow-l", dst = {{x = 448, y = 514, w = 12, h = 12}}},
		{id = "arrow-r", dst = {{x = 1120, y = 514, w = 12, h = 12}}},
		{id = "arrow-l-active", dst = {
			{x = 448, y = 514, w = 12, h = 12}
		}, mouseRect = {x = 2, y = -164, w = 340, h = 360}},
		{id = "arrow-r-active", dst = {
			{x = 1120, y = 514, w = 12, h = 12}
		}, mouseRect = {x = -330, y = -164, w = 340, h = 360}},
	}

	-- Custom property rows (6 slots, stacked from bottom).
	local custom_base_y = 252
	for slot = 1, 6 do
		local row_y = custom_base_y - (slot - 1) * 48
		-- Click area for cycling property value.
		table.insert(skin.destination, {id = "button-custom-" .. slot, dst = {
			{x = 780, y = row_y, w = 440, h = 48}
		}})
		-- Label.
		table.insert(skin.destination, {id = "custom-label-" .. slot, dst = {
			{x = 720, y = row_y + 12, w = 400, h = 24}
		}})
		-- Value.
		table.insert(skin.destination, {id = "custom-value-" .. slot, dst = {
			{x = 1000, y = row_y + 12, w = 400, h = 24}
		}})
		-- Left/right arrows for property.
		table.insert(skin.destination, {id = "arrow-l", dst = {{x = 788, y = row_y + 18, w = 12, h = 12}}})
		table.insert(skin.destination, {id = "arrow-r", dst = {{x = 1200, y = row_y + 18, w = 12, h = 12}}})
		-- Active arrows with mouse regions.
		table.insert(skin.destination, {id = "arrow-l-active", dst = {
			{x = 788, y = row_y + 18, w = 12, h = 12}
		}, mouseRect = {x = -8, y = -18, w = 220, h = 48}})
		table.insert(skin.destination, {id = "arrow-r-active", dst = {
			{x = 1200, y = row_y + 18, w = 12, h = 12}
		}, mouseRect = {x = -200, y = -18, w = 220, h = 48}})
	end

	-- Scroll bar.
	table.insert(skin.destination, {id = "scroll-bg", dst = {{x = 1260, y = 24, w = 10, h = 264}}})
	table.insert(skin.destination, {id = "scroll-fg", blend = 2, dst = {{x = 1256, y = 260, w = 17, h = 24}}})

	-- Skin type buttons (listed from top to bottom).
	local type_start_y = 630
	for idx, tid in ipairs(skin_types) do
		local row_y = type_start_y - (idx - 1) * 30
		table.insert(skin.destination, {id = "type-" .. tid, dst = {
			{x = 0, y = row_y, w = 300, h = 30}
		}})
	end

	skin.skinSelect = {
		defaultType = 6,
		customOffsetStyle = 0,
		customPropertyCount = 6,
		sampleBMS = {}
	}

	return skin
end

return {
	header = header,
	main = main
}
