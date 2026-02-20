-- 24-key (keyboard) play skin definition for beatoraja Rust port.
-- Supports half-lane, hybrid-lane, and separate-lane geometry modes.

local function extend(dst, src_list)
	for _, v in ipairs(src_list) do
		table.insert(dst, v)
	end
end

-- Skin options for lane geometry, score graph, judge display.
local property = {
	{name = "Lane Geometry", item = {
		{name = "Half Lane", op = 920},
		{name = "Hybrid Lane", op = 922},
		{name = "Separate Lane", op = 924}
	}},
	{name = "Score Graph", item = {
		{name = "Off", op = 900},
		{name = "On", op = 901}
	}},
	{name = "Judge Count", item = {
		{name = "Off", op = 905},
		{name = "On", op = 906}
	}},
	{name = "Judge Detail", item = {
		{name = "Off", op = 910},
		{name = "EARLY/LATE", op = 911},
		{name = "+-ms", op = 912}
	}}
}

-- Option query helpers.
local function is_half()
	return skin_config.option["Lane Geometry"] == 920
end
local function is_hybrid()
	return skin_config.option["Lane Geometry"] == 922
end
local function is_separate()
	return skin_config.option["Lane Geometry"] == 924
end
local function graph_on()
	return skin_config.option["Score Graph"] == 901
end

-- Timer/value id helpers for 24-key (indices 1-9 use base offsets, 10+ use 1000-series).
local function bomb_timer(idx)
	return idx <= 9 and (50 + idx) or (1000 + idx)
end
local function hold_timer(idx)
	return idx <= 9 and (70 + idx) or (1200 + idx)
end
local function keyon_timer(idx)
	return idx <= 9 and (100 + idx) or (1400 + idx)
end
local function keyoff_timer(idx)
	return idx <= 9 and (120 + idx) or (1600 + idx)
end
local function judge_value(idx)
	return idx <= 9 and (500 + idx) or (1500 + idx)
end

local filepath = {
	{name = "Background", path = "play/background/*.png"},
	{name = "Theme", path = "keyboard/*.png"},
	{name = "Laser", path = "play/laser/*.png"},
	{name = "Lanecover", path = "play/lanecover/*.png"},
}

local header = {
	type = 16,
	name = "beatoraja default (lua)",
	w = 1280,
	h = 720,
	playstart = 1000,
	scene = 3600000,
	input = 500,
	close = 1500,
	fadeout = 1000,
	property = property,
	filepath = filepath
}

-- Key colour pattern for 24 keys: 0=white, 1=black (12-key repeating pattern).
local key_pattern = { 0, 1, 0, 1, 0, 0, 1, 0, 1, 0, 1, 0 }
local function key_type(i)
	local k = (i - 1) % 26
	if k >= 24 then return 2 end
	return key_pattern[k % 12 + 1]
end
local function key_type_scratch(i)
	local k = (i - 1) % 26
	if k == 24 then return 2
	elseif k == 25 then return 3 end
	return key_pattern[k % 12 + 1]
end

-- Draw order: white keys first, then black, then scratch.
local beam_order = { 1, 3, 5, 6, 8, 10, 12, 13, 15, 17, 18, 20, 22, 24, 2, 4, 7, 9, 11, 14, 16, 19, 21, 23, 25, 26 }

local function main()
	local parts = require("play_parts")

	local skin = {}
	for k, v in pairs(header) do
		skin[k] = v
	end

	-- Compute geometry for the selected lane mode.
	local geo = {}

	if is_half() then
		geo.lanes_x = 56;   geo.lanes_w = 560
		geo.lw = 36;        geo.lb = 0;         geo.ls = 56
		geo.nw = 36;        geo.nb = 36;        geo.ns = 56
		geo.title_align = 0
		geo.judge_x = 240;  geo.ready_x = 161;  geo.title_x = 700
		geo.bga_x = 700;    geo.bga_y = 144;    geo.bga_w = 512; geo.bga_h = 512
	end
	if is_hybrid() then
		geo.lanes_x = 40;   geo.lanes_w = 578
		geo.lw = 32;        geo.lb = 8;         geo.ls = 50
		geo.nw = 32;        geo.nb = 30;        geo.ns = 50
		geo.title_align = 0
		geo.judge_x = 240;  geo.ready_x = 154;  geo.title_x = 700
		geo.bga_x = 700;    geo.bga_y = 144;    geo.bga_w = 512; geo.bga_h = 512
	end
	if is_separate() then
		geo.lanes_x = 20;   geo.lanes_w = 950
		geo.lw = 40;        geo.lb = 32;        geo.ls = 70
		geo.nw = 40;        geo.nb = 32;        geo.ns = 70
		geo.title_align = 1
		geo.judge_x = 375;  geo.ready_x = 320;  geo.title_x = 495
		geo.bga_x = 1000;   geo.bga_y = 440;    geo.bga_w = 256; geo.bga_h = 256
	end

	-- Compute per-lane positions.
	geo.nx = {}; geo.nwid = {}; geo.center = {}
	do
		local x = geo.lanes_x + geo.ls
		local aw = (geo.nw - geo.lw) / 2
		local ab = (geo.nb - geo.lb) / 2
		for i = 1, 24 do
			if key_type(i) == 0 then
				geo.nx[i] = x - aw
				geo.nwid[i] = geo.nw
				x = x + geo.lw
			else
				geo.nx[i] = x - ab
				geo.nwid[i] = geo.nb
				x = x + geo.lb
			end
		end
		-- Scratch keys occupy the leftmost lane area.
		geo.nx[25] = geo.lanes_x;   geo.nwid[25] = geo.ls
		geo.nx[26] = geo.lanes_x;   geo.nwid[26] = geo.ls
		for i = 1, 26 do
			geo.center[i] = geo.nx[i] + geo.nwid[i] / 2
		end
	end

	-- Sources.
	skin.source = {
		{id = 0, path = "system.png"},
		{id = 1, path = "play/background/*.png"},
		{id = 2, path = "playbg.png"},
		{id = 3, path = "gauge.png"},
		{id = 4, path = "judge.png"},
		{id = 5, path = "number.png"},
		{id = 6, path = "play/laser/*.png"},
		{id = 7, path = "keyboard/*.png"},
		{id = 8, path = "close.png"},
		{id = 9, path = "scoregraph.png"},
		{id = 10, path = "bomb.png"},
		{id = 11, path = "ready.png"},
		{id = 12, path = "play/lanecover/*.png"},
		{id = 13, path = "judgedetail.png"}
	}
	skin.font = {
		{id = 0, path = "VL-Gothic-Regular.ttf"}
	}

	-- Image definitions.
	skin.image = {
		{id = "background", src = 1, x = 0, y = 0, w = 1280, h = 720},
		{id = 1, src = 2, x = 0, y = 0, w = 1280, h = 720},
		{id = 6, src = 11, x = 0, y = 0, w = 216, h = 40},
		{id = 7, src = 0, x = 0, y = 0, w = 8, h = 8},
		{id = "close1", src = 8, x = 0, y = 500, w = 640, h = 240},
		{id = "close2", src = 8, x = 0, y = 740, w = 640, h = 240},
		{id = 11, src = 0, x = 168, y = 108, w = 126, h = 303},
		{id = 12, src = 0, x = 40, y = 108, w = 126, h = 303},
		{id = 13, src = 0, x = 10, y = 10, w = 10, h = 251},
		{id = 15, src = 0, x = 16, y = 0, w = 8, h = 8},

		-- Key beams.
		{id = "keybeam-w", src = 6, x = 48, y = 0, w = 27, h = 255},
		{id = "keybeam-b", src = 6, x = 76, y = 0, w = 20, h = 255},
		{id = "keybeam-s", src = 6, x = 0, y = 0, w = 47, h = 255},
		{id = "keybeam-w-pg", src = 6, x = 145, y = 0, w = 27, h = 255},
		{id = "keybeam-b-pg", src = 6, x = 173, y = 0, w = 20, h = 255},
		{id = "keybeam-s-pg", src = 6, x = 97, y = 0, w = 47, h = 255},

		-- Notes (from keyboard theme image).
		{id = "note-w", src = 7, x = 1179, y = 405, w = 27, h = 12},
		{id = "note-b", src = 7, x = 1207, y = 405, w = 21, h = 12},
		{id = "note-su", src = 7, x = 1024, y = 400, w = 50, h = 15},
		{id = "note-sd", src = 7, x = 1074, y = 400, w = 50, h = 15},

		-- LN start / end / body / active.
		{id = "lns-w", src = 7, x = 1179, y = 457, w = 27, h = 13},
		{id = "lns-b", src = 7, x = 1207, y = 457, w = 21, h = 13},
		{id = "lns-s", src = 7, x = 1130, y = 457, w = 46, h = 12},
		{id = "lne-w", src = 7, x = 1179, y = 443, w = 27, h = 13},
		{id = "lne-b", src = 7, x = 1207, y = 443, w = 21, h = 13},
		{id = "lne-s", src = 7, x = 1130, y = 443, w = 46, h = 12},
		{id = "lnb-w", src = 7, x = 1179, y = 480, w = 27, h = 1},
		{id = "lnb-b", src = 7, x = 1207, y = 480, w = 21, h = 1},
		{id = "lnb-s", src = 7, x = 1130, y = 480, w = 46, h = 1},
		{id = "lna-w", src = 7, x = 1179, y = 476, w = 27, h = 1},
		{id = "lna-b", src = 7, x = 1207, y = 476, w = 21, h = 1},
		{id = "lna-s", src = 7, x = 1130, y = 476, w = 46, h = 1},

		-- HCN variants.
		{id = "hcns-w", src = 7, x = 1179, y = 508, w = 27, h = 13},
		{id = "hcns-b", src = 7, x = 1207, y = 508, w = 21, h = 13},
		{id = "hcns-s", src = 7, x = 1130, y = 508, w = 46, h = 12},
		{id = "hcne-w", src = 7, x = 1179, y = 494, w = 27, h = 13},
		{id = "hcne-b", src = 7, x = 1207, y = 494, w = 21, h = 13},
		{id = "hcne-s", src = 7, x = 1130, y = 494, w = 46, h = 12},
		{id = "hcnb-w", src = 7, x = 1179, y = 531, w = 27, h = 1},
		{id = "hcnb-b", src = 7, x = 1207, y = 531, w = 21, h = 1},
		{id = "hcnb-s", src = 7, x = 1130, y = 531, w = 46, h = 1},
		{id = "hcna-w", src = 7, x = 1179, y = 527, w = 27, h = 1},
		{id = "hcna-b", src = 7, x = 1207, y = 527, w = 21, h = 1},
		{id = "hcna-s", src = 7, x = 1130, y = 527, w = 46, h = 1},
		{id = "hcnd-w", src = 7, x = 1179, y = 528, w = 27, h = 1},
		{id = "hcnd-b", src = 7, x = 1207, y = 528, w = 21, h = 1},
		{id = "hcnd-s", src = 7, x = 1130, y = 528, w = 46, h = 1},
		{id = "hcnr-w", src = 7, x = 1179, y = 529, w = 27, h = 1},
		{id = "hcnr-b", src = 7, x = 1207, y = 529, w = 21, h = 1},
		{id = "hcnr-s", src = 7, x = 1130, y = 529, w = 46, h = 1},

		-- Mines.
		{id = "mine-w", src = 7, x = 1179, y = 423, w = 27, h = 8},
		{id = "mine-b", src = 7, x = 1207, y = 423, w = 21, h = 8},
		{id = "mine-s", src = 7, x = 1130, y = 423, w = 46, h = 8},

		{id = "section-line", src = 0, x = 0, y = 0, w = 1, h = 1},

		-- Gauge sprites (normal / expert).
		{id = "gauge-n1", src = 3, x = 0, y = 0, w = 5, h = 17},
		{id = "gauge-n2", src = 3, x = 5, y = 0, w = 5, h = 17},
		{id = "gauge-n3", src = 3, x = 10, y = 0, w = 5, h = 17},
		{id = "gauge-n4", src = 3, x = 15, y = 0, w = 5, h = 17},
		{id = "gauge-e1", src = 3, x = 0, y = 17, w = 5, h = 17},
		{id = "gauge-e2", src = 3, x = 5, y = 17, w = 5, h = 17},
		{id = "gauge-e3", src = 3, x = 10, y = 17, w = 5, h = 17},
		{id = "gauge-e4", src = 3, x = 15, y = 17, w = 5, h = 17},

		-- Judge word sprites.
		{id = "judgef-pg", src = 4, x = 0, y = 0, w = 180, h = 100, divy = 2, cycle = 100},
		{id = "judgef-gr", src = 4, x = 0, y = 150, w = 180, h = 50},
		{id = "judgef-gd", src = 4, x = 0, y = 200, w = 180, h = 50},
		{id = "judgef-bd", src = 4, x = 0, y = 250, w = 180, h = 50},
		{id = "judgef-pr", src = 4, x = 0, y = 300, w = 180, h = 50},
		{id = "judgef-ms", src = 4, x = 0, y = 300, w = 180, h = 50},

		-- Early/late.
		{id = "judge-early", src = 13, x = 0, y = 0, w = 50, h = 20},
		{id = "judge-late", src = 13, x = 50, y = 0, w = 50, h = 20}
	}

	-- Bomb / hold animations for all 26 keys.
	local function make_bomb(idx, prefix, sy, timer_fn)
		local label = idx
		if idx == 25 then label = "su"
		elseif idx == 26 then label = "sd" end
		return {id = prefix .. label, src = 10, x = 0, y = sy, w = 1810, h = 192, divx = 10, timer = timer_fn(idx), cycle = 160}
	end
	for i = 1, 26 do
		table.insert(skin.image, make_bomb(i, "bomb1-", 0, bomb_timer))
		table.insert(skin.image, make_bomb(i, "bomb2-", 576, bomb_timer))
		table.insert(skin.image, make_bomb(i, "bomb3-", 192, bomb_timer))
		table.insert(skin.image, make_bomb(i, "hold-", 384, hold_timer))
	end

	-- Lane background and key images depend on geometry mode.
	if is_half() then
		table.insert(skin.image, {id = "lane-bg", src = 7, x = 56, y = 0, w = 560, h = 80})
		table.insert(skin.image, {id = "keys", src = 7, x = 56, y = 100, w = 560, h = 80})
	end
	if is_hybrid() then
		table.insert(skin.image, {id = "lane-bg", src = 7, x = 40, y = 200, w = 578, h = 80})
		table.insert(skin.image, {id = "keys", src = 7, x = 40, y = 300, w = 578, h = 80})
	end
	if is_separate() then
		table.insert(skin.image, {id = "lane-bg", src = 7, x = 0, y = 400, w = 950, h = 80})
		table.insert(skin.image, {id = "keys", src = 7, x = 0, y = 480, w = 950, h = 80})
	end

	-- Imagesets for key beams and bomb selection.
	skin.imageset = {}
	do
		local suffixes = { "w", "b", "s" }
		for i = 1, 26 do
			local label = i
			if i == 25 then label = "su"
			elseif i == 26 then label = "sd" end
			local s = suffixes[key_type(i) + 1]
			table.insert(skin.imageset, {
				id = "keybeam" .. label,
				ref = judge_value(i),
				images = { "keybeam-" .. s, "keybeam-" .. s .. "-pg" }
			})
		end
	end
	for i = 1, 26 do
		local label = i
		if i == 25 then label = "su"
		elseif i == 26 then label = "sd" end
		table.insert(skin.imageset, {
			id = i + 109,
			ref = judge_value(i),
			images = { "bomb1-" .. label, "bomb2-" .. label, "bomb1-" .. label, "bomb3-" .. label }
		})
	end

	-- Numeric values.
	skin.value = {
		{id = 400, src = 5, x = 0, y = 0, w = 240, h = 24, divx = 10, digit = 4, ref = 91},
		{id = 401, src = 5, x = 0, y = 0, w = 240, h = 24, divx = 10, digit = 4, ref = 160},
		{id = 402, src = 5, x = 0, y = 0, w = 240, h = 24, divx = 10, digit = 4, ref = 90},
		{id = 403, src = 5, x = 0, y = 0, w = 240, h = 24, divx = 10, digit = 2, ref = 163},
		{id = 404, src = 5, x = 0, y = 0, w = 240, h = 24, divx = 10, digit = 2, padding = 1, ref = 164},
		{id = 405, src = 5, x = 0, y = 0, w = 240, h = 24, divx = 10, digit = 2, ref = 310},
		{id = 406, src = 5, x = 0, y = 0, w = 240, h = 24, divx = 10, digit = 2, padding = 1, ref = 311},
		{id = 407, src = 5, x = 0, y = 0, w = 240, h = 24, divx = 10, digit = 4, ref = 312},
		{id = 410, src = 5, x = 0, y = 0, w = 240, h = 24, divx = 10, digit = 3, ref = 107},
		{id = 411, src = 5, x = 0, y = 0, w = 240, h = 24, divx = 10, digit = 1, ref = 407},
		{id = 420, src = 5, x = 0, y = 0, w = 240, h = 24, divx = 10, digit = 4, ref = 102},
		{id = 421, src = 5, x = 0, y = 0, w = 240, h = 24, divx = 10, digit = 1, ref = 103},
		{id = 422, src = 5, x = 0, y = 0, w = 240, h = 24, divx = 10, digit = 5, ref = 71},
		{id = 423, src = 5, x = 0, y = 24, w = 240, h = 24, divx = 10, digit = 5, ref = 150},
		{id = 424, src = 5, x = 0, y = 48, w = 240, h = 24, divx = 10, digit = 5, ref = 121},

		{id = 450, src = 0, x = 0, y = 550, w = 100, h = 15, divx = 10, digit = 3, ref = 14},
		{id = 451, src = 0, x = 0, y = 565, w = 100, h = 15, divx = 10, digit = 4, ref = 312},

		-- Judge combo numbers.
		{id = "judgen-pg", src = 4, x = 200, y = 0, w = 300, h = 100, divx = 10, divy = 2, digit = 6, ref = 75, cycle = 100},
		{id = "judgen-gr", src = 4, x = 200, y = 150, w = 300, h = 50, divx = 10, digit = 6, ref = 75},
		{id = "judgen-gd", src = 4, x = 200, y = 200, w = 300, h = 50, divx = 10, digit = 6, ref = 75},
		{id = "judgen-bd", src = 4, x = 200, y = 250, w = 300, h = 50, divx = 10, digit = 6, ref = 75},
		{id = "judgen-pr", src = 4, x = 200, y = 300, w = 300, h = 50, divx = 10, digit = 6, ref = 75},
		{id = "judgen-ms", src = 4, x = 200, y = 300, w = 300, h = 50, divx = 10, digit = 6, ref = 75},

		-- Timing ms.
		{id = "judgems-1pp", src = 13, x = 0, y = 20, w = 120, h = 40, divx = 12, divy = 2, digit = 4, ref = 525},
		{id = "judgems-1pg", src = 13, x = 0, y = 60, w = 120, h = 40, divx = 12, divy = 2, digit = 4, ref = 525}
	}
	extend(skin.value, parts.judge_count_sources("judge-count-", 5))

	skin.text = {
		{id = 1000, font = 0, size = 24, align = geo.title_align, ref = 12}
	}

	skin.slider = {
		{id = 1050, src = 0, x = 0, y = 289, w = 14, h = 20, angle = 2, range = 520, type = 6},
		{id = 1051, src = 0, x = 15, y = 289, w = 14, h = 20, angle = 2, range = 520, type = 6},
		{id = 1060, src = 12, x = 0, y = 0, w = 390, h = 580, angle = 2, range = 580, type = 4}
	}

	skin.hiddenCover = {
		{id = "hidden-cover", src = 12, x = 0, y = 0, w = 390, h = 580, disapearLine = 140, isDisapearLineLinkLift = true}
	}

	skin.graph = {
		{id = "graph-now", src = 9, x = 0, y = 0, w = 100, h = 296, type = 110},
		{id = "graph-best", src = 9, x = 100, y = 0, w = 100, h = 296, type = 113},
		{id = "graph-target", src = 9, x = 200, y = 0, w = 100, h = 296, type = 115},
		{id = "load-progress", src = 0, x = 0, y = 0, w = 8, h = 8, angle = 0, type = 102}
	}

	-- Build 24+2 key note arrays.
	-- Pattern: keys 1-24 follow the 12-key W/B cycle, keys 25-26 are scratch.
	local function make_note_array(prefix_w, prefix_b, prefix_s)
		local t = {}
		for i = 1, 24 do
			t[i] = key_type(i) == 0 and prefix_w or prefix_b
		end
		t[25] = prefix_s; t[26] = prefix_s
		return t
	end

	skin.note = {
		id = "notes",
		note = (function()
			local t = {}
			for i = 1, 24 do
				t[i] = key_type(i) == 0 and "note-w" or "note-b"
			end
			t[25] = "note-su"; t[26] = "note-sd"
			return t
		end)(),
		lnstart = make_note_array("lns-w", "lns-b", "lns-s"),
		lnend = make_note_array("lne-w", "lne-b", "lne-s"),
		lnbody = make_note_array("lnb-w", "lnb-b", "lnb-s"),
		lnactive = make_note_array("lna-w", "lna-b", "lna-s"),
		hcnstart = make_note_array("hcns-w", "hcns-b", "hcns-s"),
		hcnend = make_note_array("hcne-w", "hcne-b", "hcne-s"),
		hcnbody = make_note_array("hcnb-w", "hcnb-b", "hcnb-s"),
		hcnactive = make_note_array("hcna-w", "hcna-b", "hcna-s"),
		hcndamage = make_note_array("hcnd-w", "hcnd-b", "hcnd-s"),
		hcnreactive = make_note_array("hcnr-w", "hcnr-b", "hcnr-s"),
		mine = make_note_array("mine-w", "mine-b", "mine-s"),
		hidden = {},
		processed = {},
		group = {
			{id = "section-line", offset = 50, dst = {
				{x = geo.lanes_x, y = 140, w = geo.lanes_w, h = 1, r = 128, g = 128, b = 128}
			}}
		},
		time = {
			{id = "section-line", offset = 50, dst = {
				{x = geo.lanes_x, y = 140, w = geo.lanes_w, h = 1, r = 64, g = 192, b = 192}
			}}
		},
		bpm = {
			{id = "section-line", offset = 50, dst = {
				{x = geo.lanes_x, y = 140, w = geo.lanes_w, h = 2, r = 0, g = 192, b = 0}
			}}
		},
		stop = {
			{id = "section-line", offset = 50, dst = {
				{x = geo.lanes_x, y = 140, w = geo.lanes_w, h = 2, r = 192, g = 192, b = 0}
			}}
		}
	}
	skin.note.dst = {}
	for i = 1, 26 do
		skin.note.dst[i] = {
			x = geo.nx[i], y = 140, w = geo.nwid[i], h = 580
		}
	end

	-- Gauge.
	skin.gauge = {
		id = 2001,
		nodes = {"gauge-n1","gauge-n2","gauge-n3","gauge-n4","gauge-e1","gauge-e2","gauge-e3","gauge-e4"}
	}

	-- Judge display.
	skin.judge = {
		{
			id = 2010,
			index = 0,
			images = {
				{id = "judgef-pg", loop = -1, timer = 46, offset = 50, dst = {
					{time = 0, x = geo.judge_x, y = 240, w = 180, h = 40},
					{time = 500}
				}},
				{id = "judgef-gr", loop = -1, timer = 46, offset = 50, dst = {
					{time = 0, x = geo.judge_x, y = 240, w = 180, h = 40},
					{time = 500}
				}},
				{id = "judgef-gd", loop = -1, timer = 46, offset = 50, dst = {
					{time = 0, x = geo.judge_x, y = 240, w = 180, h = 40},
					{time = 500}
				}},
				{id = "judgef-bd", loop = -1, timer = 46, offset = 50, dst = {
					{time = 0, x = geo.judge_x, y = 240, w = 180, h = 40},
					{time = 500}
				}},
				{id = "judgef-pr", loop = -1, timer = 46, offset = 50, dst = {
					{time = 0, x = geo.judge_x, y = 240, w = 180, h = 40},
					{time = 500}
				}},
				{id = "judgef-ms", loop = -1, timer = 46, offset = 50, dst = {
					{time = 0, x = geo.judge_x, y = 240, w = 180, h = 40},
					{time = 500}
				}}
			},
			numbers = {
				{id = "judgen-pg", loop = -1, timer = 46, dst = {
					{time = 0, x = 200, y = 0, w = 40, h = 40},
					{time = 500}
				}},
				{id = "judgen-gr", loop = -1, timer = 46, dst = {
					{time = 0, x = 200, y = 0, w = 40, h = 40},
					{time = 500}
				}},
				{id = "judgen-gd", loop = -1, timer = 46, dst = {
					{time = 0, x = 200, y = 0, w = 40, h = 40},
					{time = 500}
				}},
				{id = "judgen-bd", loop = -1, timer = 46, dst = {
					{time = 0, x = 200, y = 0, w = 40, h = 40},
					{time = 500}
				}},
				{id = "judgen-pr", loop = -1, timer = 46, dst = {
					{time = 0, x = 200, y = 0, w = 40, h = 40},
					{time = 500}
				}},
				{id = "judgen-ms", loop = -1, timer = 46, dst = {
					{time = 0, x = 200, y = 0, w = 40, h = 40},
					{time = 500}
				}}
			},
			shift = true
		}
	}

	skin.bga = { id = 2002 }

	-- Destinations.
	skin.destination = {
		{id = "background", dst = {
			{x = 0, y = 0, w = 1280, h = 720}
		}},
		{id = 1, dst = {
			{x = 0, y = 0, w = 1280, h = 720}
		}},

		-- BPM displays.
		{id = 400, dst = {{x = 520, y = 2, w = 18, h = 18}}},
		{id = 401, dst = {{x = 592, y = 2, w = 24, h = 24}}},
		{id = 402, dst = {{x = 688, y = 2, w = 18, h = 18}}},
		-- Time left.
		{id = 403, dst = {{x = 1148, y = 2, w = 24, h = 24}}},
		{id = 404, dst = {{x = 1220, y = 2, w = 24, h = 24}}},
		-- Hi-speed.
		{id = 405, dst = {{x = 116, y = 2, w = 12, h = 24}}},
		{id = 406, dst = {{x = 154, y = 2, w = 10, h = 20}}},
		-- Duration.
		{id = 407, dst = {{x = 318, y = 2, w = 12, h = 24}}},

		-- Progress bar track.
		{id = 13, dst = {{x = 4, y = 140, w = 12, h = 540}}},

		-- Lane background.
		{id = "lane-bg", loop = 1000, dst = {
			{time = 0, x = geo.lanes_x, y = 140, w = geo.lanes_w, h = 0, a = 0},
			{time = 1000, h = 580, a = 255}
		}},
		{id = "keys", dst = {
			{x = geo.lanes_x, y = 100, w = geo.lanes_w, h = 80}
		}}
	}

	-- Key beams.
	for _, i in ipairs(beam_order) do
		local label = i
		if i == 25 then label = "su"
		elseif i == 26 then label = "sd" end
		table.insert(skin.destination, {
			id = "keybeam" .. label,
			timer = keyon_timer(i),
			loop = 100,
			offset = 50,
			dst = {
				{time = 0, x = geo.nx[i] + geo.nwid[i] / 4, y = 140, w = geo.nwid[i] / 2, h = 580},
				{time = 100, x = geo.nx[i], w = geo.nwid[i]}
			}
		})
	end

	-- Judge line.
	table.insert(skin.destination, {id = 15, offset = 50, dst = {{x = geo.lanes_x, y = 137, w = geo.lanes_w, h = 6}}})

	-- Notes.
	table.insert(skin.destination, {id = "notes"})

	-- Bomb effects.
	for i = 1, 26 do
		table.insert(skin.destination, {
			id = 109 + i,
			timer = bomb_timer(i),
			blend = 2,
			loop = -1,
			offset = 50,
			dst = {
				{time = 0, x = geo.center[i] - 80, y = 28, w = 180, h = 192},
				{time = 160}
			}
		})
	end

	-- Hold effects.
	for i = 1, 26 do
		local label = i
		if i == 25 then label = "su"
		elseif i == 26 then label = "sd" end
		table.insert(skin.destination, {
			id = "hold-" .. label,
			timer = hold_timer(i),
			blend = 2,
			offset = 50,
			dst = {
				{time = 0, x = geo.center[i] - 80, y = 28, w = 180, h = 192}
			}
		})
	end

	-- Judge, early/late, covers, gauge.
	extend(skin.destination, {
		{id = 2010},
		{id = "judge-early", loop = -1, timer = 46, op = {911, 1242}, offset = 50, dst = {
			{time = 0, x = 320, y = 290, w = 50, h = 20},
			{time = 500}
		}},
		{id = "judge-late", loop = -1, timer = 46, op = {911, 1243}, offset = 50, dst = {
			{time = 0, x = 320, y = 290, w = 50, h = 20},
			{time = 500}
		}},
		{id = "judgems-1pp", loop = -1, timer = 46, op = {912, 241}, offset = 50, dst = {
			{time = 0, x = 320, y = 290, w = 10, h = 20},
			{time = 500}
		}},
		{id = "judgems-1pg", loop = -1, timer = 46, op = {912, -241}, offset = 50, dst = {
			{time = 0, x = 320, y = 290, w = 10, h = 20},
			{time = 500}
		}},
		{id = "hidden-cover", dst = {
			{x = geo.lanes_x, y = -440, w = geo.lanes_w, h = 580}
		}},
		{id = 1060, dst = {
			{x = geo.lanes_x, y = 720, w = geo.lanes_w, h = 580}
		}},
		{id = 2001, dst = {
			{time = 0, x = 20, y = 30, w = 390, h = 30}
		}},
		{id = 410, dst = {
			{time = 0, x = 314, y = 60, w = 24, h = 24}
		}},
		{id = 411, dst = {
			{time = 0, x = 386, y = 60, w = 18, h = 18}
		}}
	})

	-- BGA, title, score graph, progress.
	extend(skin.destination, {
		{id = 2002, dst = {
			{time = 0, x = geo.bga_x, y = geo.bga_y, w = geo.bga_w, h = geo.bga_h}
		}},
		{id = 1000, dst = {
			{time = 0, x = geo.title_x, y = 674, w = 24, h = 24},
			{time = 1000, a = 0},
			{time = 2000, a = 255}
		}},
		{id = 11, op = {901}, dst = {
			{x = 1132, y = 50, w = 120, h = 360}
		}},
		{id = "graph-now", op = {901}, dst = {
			{x = 1133, y = 50, w = 38, h = 360}
		}},
		{id = "graph-best", op = {901}, dst = {
			{x = 1173, y = 50, w = 38, h = 360}
		}},
		{id = "graph-target", op = {901}, dst = {
			{x = 1213, y = 50, w = 38, h = 360}
		}},
		{id = 12, op = {901}, dst = {
			{x = 1132, y = 50, w = 120, h = 360}
		}},
		{id = 420, op = {901}, dst = {
			{x = 1020, y = 230, w = 12, h = 18}
		}},
		{id = 421, op = {901}, dst = {
			{x = 1068, y = 230, w = 8, h = 12}
		}},
		{id = 422, op = {901}, dst = {
			{x = 1020, y = 210, w = 12, h = 18}
		}},
		{id = 423, op = {901}, dst = {
			{x = 1020, y = 190, w = 12, h = 18}
		}},
		{id = 424, op = {901}, dst = {
			{x = 1020, y = 170, w = 12, h = 18}
		}},
		{id = 1050, blend = 2, dst = {
			{x = 2, y = 660, w = 16, h = 20}
		}},
		{id = 1051, blend = 2, timer = 143, dst = {
			{x = 2, y = 660, w = 16, h = 20}
		}},
	})

	-- Judge count overlays.
	extend(skin.destination, parts.judge_count_destinations("judge-count-", 1000, 50, {906}, -1))

	-- Lane cover values, loading progress, ready, close, fade.
	extend(skin.destination, {
		{id = 450, offset = 51, op = {270}, dst = {
			{time = 0, x = 120, y = 720, w = 10, h = 15}
		}},
		{id = 451, offset = 51, op = {270}, dst = {
			{time = 0, x = 310, y = 720, w = 10, h = 15}
		}},
		{id = "load-progress", loop = 0, op = {80}, dst = {
			{time = 0, x = geo.lanes_x, y = 440, w = geo.lanes_w, h = 4},
			{time = 500, a = 192, r = 0},
			{time = 1000, a = 128, r = 255, g = 0},
			{time = 1500, a = 192, g = 255, b = 0},
			{time = 2000, a = 255, b = 255}
		}},
		{id = 6, loop = -1, timer = 40, dst = {
			{time = 0, x = geo.ready_x, y = 250, w = 350, h = 60, a = 0},
			{time = 750, y = 300, a = 255},
			{time = 1000}
		}},
		{id = "close2", loop = 700, timer = 3, dst = {
			{time = 0, x = 0, y = -360, w = 1280, h = 360},
			{time = 500, y = 0},
			{time = 600, y = -40},
			{time = 700, y = 0}
		}},
		{id = "close1", loop = 700, timer = 3, dst = {
			{time = 0, x = 0, y = 720, w = 1280, h = 360},
			{time = 500, y = 360},
			{time = 600, y = 400},
			{time = 700, y = 360}
		}},
		{id = 7, loop = 500, timer = 2, dst = {
			{time = 0, x = 0, y = 0, w = 1280, h = 720, a = 0},
			{time = 500, a = 255}
		}}
	})

	return skin
end

return {
	header = header,
	main = main
}
