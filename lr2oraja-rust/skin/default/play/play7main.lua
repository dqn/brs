-- 7-key play skin definition for beatoraja Rust port.
-- Provides header metadata and full skin layout via main().

local function extend(dst, src_list)
	for _, v in ipairs(src_list) do
		table.insert(dst, v)
	end
end

-- Skin property options exposed to the user.
local property = {
	{name = "Play Side", item = {
		{name = "1P", op = 920},
		{name = "2P", op = 921}
	}},
	{name = "Scratch Side", item = {
		{name = "Left", op = 922},
		{name = "Right", op = 923}
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
local function side_1p()
	return skin_config.option["Play Side"] == 920
end
local function side_2p()
	return skin_config.option["Play Side"] == 921
end
local function scratch_left()
	return skin_config.option["Scratch Side"] == 922
end
local function scratch_right()
	return skin_config.option["Scratch Side"] == 923
end
local function graph_on()
	return skin_config.option["Score Graph"] == 901
end
local function judge_count_on()
	return skin_config.option["Judge Count"] == 906
end

-- Timer id helpers for key indices (index 8 = scratch).
local function bomb_timer(idx)
	return idx == 8 and 50 or (50 + idx)
end
local function hold_timer(idx)
	return idx == 8 and 70 or (70 + idx)
end
local function keyon_timer(idx)
	return idx == 8 and 100 or (100 + idx)
end
local function keyoff_timer(idx)
	return idx == 8 and 120 or (120 + idx)
end
local function judge_value(idx)
	return idx == 8 and 500 or (500 + idx)
end

-- File path selector options.
local filepath = {
	{name = "Background", path = "background/*.png"},
	{name = "Note", path = "notes/*.png"},
	{name = "Bomb", path = "bomb/*.png"},
	{name = "Gauge", path = "gauge/*.png"},
	{name = "Judge", path = "judge/*.png"},
	{name = "Laser", path = "laser/*.png"},
	{name = "Lanecover", path = "lanecover/*.png"},
}

-- Offset definitions for user-adjustable offsets.
local offset = {
	{name = "Laser Offset", id = 40, x = false, y = false, w = false, h = true, r = false, a = true},
	{name = "Bomb Offset", id = 41, x = true, y = true, w = true, h = true, r = false, a = true},
	{name = "Judge Count Offset", id = 42, x = true, y = true, w = false, h = false, r = false, a = true},
	{name = "BGA Offset", id = 43, x = true, y = true, w = true, h = true, r = false, a = true},
	{name = "Lane Background Offset", id = 44, x = false, y = false, w = false, h = false, r = false, a = true},
}

-- Skin header (returned when skin_config is nil).
local header = {
	type = 0,
	name = "beatoraja default (lua)",
	w = 1280,
	h = 720,
	playstart = 1000,
	scene = 3600000,
	input = 500,
	close = 1500,
	fadeout = 1000,
	property = property,
	filepath = filepath,
	offset = offset
}

-- Key colour pattern: 0=white, 1=black, 2=scratch.
local key_colours = { 0, 1, 0, 1, 0, 1, 0, 2 }
local function key_type(i)
	return key_colours[(i - 1) % 8 + 1]
end

-- Draw order for key beams (all 8 keys).
local beam_order = { 1, 2, 3, 4, 5, 6, 7, 8 }

local function main()
	local parts = require("play_parts")

	-- Copy header fields into the skin table.
	local skin = {}
	for k, v in pairs(header) do
		skin[k] = v
	end

	-- Compute layout geometry based on options.
	local geo = {}

	if side_1p() then
		geo.lanes_x = 20;   geo.lanes_w = 390
		geo.lw = 50;        geo.lb = 40;        geo.ls = 70
		geo.nw = 50;        geo.nb = 40;        geo.ns = 70
		geo.title_align = 0
		geo.judge_x = 115;  geo.jdetail_x = 200; geo.jdetail_y = 290
		geo.jcount_x = 476; geo.jcount_y = 50
		geo.ready_x = 40;   geo.title_x = 450
		geo.gauge_x = 20;   geo.gauge_w = 390
		geo.gval_x = 314
		geo.bga_x = 440;    geo.bga_y = 50;   geo.bga_w = 800; geo.bga_h = 650
		geo.jgraph_x = 740; geo.jgraph_y = 100; geo.jgraph_w = 450; geo.jgraph_h = 100
		geo.timing_x = 740; geo.timing_y = 50;  geo.timing_w = 450; geo.timing_h = 50
		geo.prog_x = 2;     geo.prog_y = 140;   geo.prog_w = 16;   geo.prog_h = 540
	end
	if side_2p() then
		geo.lanes_x = 870;  geo.lanes_w = 390
		geo.lw = 50;        geo.lb = 40;        geo.ls = 70
		geo.nw = 50;        geo.nb = 40;        geo.ns = 70
		geo.title_align = 2
		geo.judge_x = 965;  geo.jdetail_x = 1050; geo.jdetail_y = 290
		geo.jcount_x = 720; geo.jcount_y = 50
		geo.ready_x = 890;  geo.title_x = 840
		geo.gauge_x = 1260;  geo.gauge_w = -390
		geo.gval_x = 870
		geo.bga_x = 40;     geo.bga_y = 50;   geo.bga_w = 800; geo.bga_h = 650
		geo.jgraph_x = 90;  geo.jgraph_y = 100; geo.jgraph_w = 450; geo.jgraph_h = 100
		geo.timing_x = 90;  geo.timing_y = 50;  geo.timing_w = 450; geo.timing_h = 50
		geo.prog_x = 1262;  geo.prog_y = 140;   geo.prog_w = 16;   geo.prog_h = 540
	end

	-- Score graph area adjustment.
	if graph_on() then
		if side_1p() then
			geo.graph_x = geo.lanes_x + geo.lanes_w
			geo.title_x = geo.title_x + 90
			geo.bga_x = geo.bga_x + 90
			geo.bga_w = geo.bga_w - 90
			geo.jcount_x = geo.jcount_x + 90
		else
			geo.graph_x = geo.lanes_x - 90
			geo.title_x = geo.title_x - 90
			geo.bga_w = geo.bga_w - 90
			geo.jcount_x = geo.jcount_x - 90
		end
		geo.graph_y = 220; geo.graph_w = 90; geo.graph_h = 480
	else
		geo.graph_x = 0; geo.graph_y = 0; geo.graph_w = 0; geo.graph_h = 0
	end

	-- Per-lane note x positions and widths.
	geo.nx = {}; geo.nwid = {}; geo.center = {}
	do
		local x = geo.lanes_x
		if scratch_left() then
			geo.lanebg_x = geo.lanes_x
			geo.lanebg_w = geo.lanes_w
			x = x + geo.ls
			geo.nx[8] = geo.lanes_x
			geo.nwid[8] = geo.ls
		end
		local aw = (geo.nw - geo.lw) / 2
		local ab = (geo.nb - geo.lb) / 2
		for i = 1, 7 do
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
		if scratch_right() then
			geo.lanebg_x = geo.lanes_x + geo.lanes_w
			geo.lanebg_w = -geo.lanes_w
			geo.nx[8] = x
			geo.nwid[8] = geo.ls
		end
		for i = 1, 8 do
			geo.center[i] = geo.nx[i] + geo.nwid[i] / 2
		end
	end

	-- Image sources.
	skin.source = {
		{id = 0, path = "../system.png"},
		{id = "bg", path = "background/*.png"},
		{id = 2, path = "../playbg.png"},
		{id = 3, path = "gauge/*.png"},
		{id = 4, path = "judge/*.png"},
		{id = 5, path = "../number.png"},
		{id = 6, path = "laser/*.png"},
		{id = 7, path = "notes/*.png"},
		{id = 8, path = "../close.png"},
		{id = 9, path = "../scoregraph.png"},
		{id = 10, path = "bomb/*.png"},
		{id = 11, path = "../ready.png"},
		{id = 12, path = "lanecover/*.png"},
		{id = 13, path = "../judgedetail.png"}
	}
	skin.font = {
		{id = 0, path = "../VL-Gothic-Regular.ttf"}
	}

	-- Sprite / image definitions.
	skin.image = {
		{id = "background", src = "bg", x = 0, y = 0, w = 1280, h = 720},
		{id = 1, src = 2, x = 0, y = 0, w = 1280, h = 720},
		{id = "ready", src = 11, x = 0, y = 0, w = 216, h = 40},
		{id = 7, src = 0, x = 0, y = 0, w = 8, h = 8},
		{id = "close1", src = 8, x = 0, y = 500, w = 640, h = 240},
		{id = "close2", src = 8, x = 0, y = 740, w = 640, h = 240},
		{id = "lane-bg", src = 0, x = 30, y = 0, w = 390, h = 10},
		{id = 11, src = 0, x = 168, y = 108, w = 126, h = 303},
		{id = 12, src = 0, x = 40, y = 108, w = 126, h = 303},
		{id = 13, src = 0, x = 10, y = 10, w = 10, h = 251},
		{id = 15, src = 0, x = 16, y = 0, w = 8, h = 8},

		-- Key beam sprites (normal + pgreat).
		{id = "keybeam-w", src = 6, x = 48, y = 0, w = 27, h = 255},
		{id = "keybeam-b", src = 6, x = 76, y = 0, w = 20, h = 255},
		{id = "keybeam-s", src = 6, x = 0, y = 0, w = 47, h = 255},
		{id = "keybeam-w-pg", src = 6, x = 145, y = 0, w = 27, h = 255},
		{id = "keybeam-b-pg", src = 6, x = 173, y = 0, w = 20, h = 255},
		{id = "keybeam-s-pg", src = 6, x = 97, y = 0, w = 47, h = 255},

		-- Note sprites.
		{id = "note-w", src = 7, x = 99, y = 5, w = 27, h = 12},
		{id = "note-b", src = 7, x = 127, y = 5, w = 21, h = 12},
		{id = "note-s", src = 7, x = 50, y = 5, w = 46, h = 12},

		-- LN start.
		{id = "lns-w", src = 7, x = 99, y = 57, w = 27, h = 13},
		{id = "lns-b", src = 7, x = 127, y = 57, w = 21, h = 13},
		{id = "lns-s", src = 7, x = 50, y = 57, w = 46, h = 12},

		-- LN end.
		{id = "lne-w", src = 7, x = 99, y = 43, w = 27, h = 13},
		{id = "lne-b", src = 7, x = 127, y = 43, w = 21, h = 13},
		{id = "lne-s", src = 7, x = 50, y = 43, w = 46, h = 12},

		-- LN body / active.
		{id = "lnb-w", src = 7, x = 99, y = 80, w = 27, h = 1},
		{id = "lnb-b", src = 7, x = 127, y = 80, w = 21, h = 1},
		{id = "lnb-s", src = 7, x = 50, y = 80, w = 46, h = 1},
		{id = "lna-w", src = 7, x = 99, y = 76, w = 27, h = 1},
		{id = "lna-b", src = 7, x = 127, y = 76, w = 21, h = 1},
		{id = "lna-s", src = 7, x = 50, y = 76, w = 46, h = 1},

		-- HCN variants.
		{id = "hcns-w", src = 7, x = 99, y = 108, w = 27, h = 13},
		{id = "hcns-b", src = 7, x = 127, y = 108, w = 21, h = 13},
		{id = "hcns-s", src = 7, x = 50, y = 108, w = 46, h = 12},
		{id = "hcne-w", src = 7, x = 99, y = 94, w = 27, h = 13},
		{id = "hcne-b", src = 7, x = 127, y = 94, w = 21, h = 13},
		{id = "hcne-s", src = 7, x = 50, y = 94, w = 46, h = 12},
		{id = "hcnb-w", src = 7, x = 99, y = 131, w = 27, h = 1},
		{id = "hcnb-b", src = 7, x = 127, y = 131, w = 21, h = 1},
		{id = "hcnb-s", src = 7, x = 50, y = 131, w = 46, h = 1},
		{id = "hcna-w", src = 7, x = 99, y = 127, w = 27, h = 1},
		{id = "hcna-b", src = 7, x = 127, y = 127, w = 21, h = 1},
		{id = "hcna-s", src = 7, x = 50, y = 127, w = 46, h = 1},
		{id = "hcnd-w", src = 7, x = 99, y = 128, w = 27, h = 1},
		{id = "hcnd-b", src = 7, x = 127, y = 128, w = 21, h = 1},
		{id = "hcnd-s", src = 7, x = 50, y = 128, w = 46, h = 1},
		{id = "hcnr-w", src = 7, x = 99, y = 129, w = 27, h = 1},
		{id = "hcnr-b", src = 7, x = 127, y = 129, w = 21, h = 1},
		{id = "hcnr-s", src = 7, x = 50, y = 129, w = 46, h = 1},

		-- Mine notes.
		{id = "mine-w", src = 7, x = 99, y = 23, w = 27, h = 8},
		{id = "mine-b", src = 7, x = 127, y = 23, w = 21, h = 8},
		{id = "mine-s", src = 7, x = 50, y = 23, w = 46, h = 8},

		-- Section line pixel.
		{id = "section-line", src = 0, x = 0, y = 0, w = 1, h = 1},

		-- Gauge sprites (red/blue/yellow/purple, on/off/border).
		{id = "gauge-r1", src = 3, x = 0, y = 0, w = 5, h = 17},
		{id = "gauge-b1", src = 3, x = 5, y = 0, w = 5, h = 17},
		{id = "gauge-r2", src = 3, x = 10, y = 0, w = 5, h = 17},
		{id = "gauge-b2", src = 3, x = 15, y = 0, w = 5, h = 17},
		{id = "gauge-r3", src = 3, x = 0, y = 34, w = 5, h = 17},
		{id = "gauge-b3", src = 3, x = 5, y = 34, w = 5, h = 17},
		{id = "gauge-y1", src = 3, x = 0, y = 17, w = 5, h = 17},
		{id = "gauge-p1", src = 3, x = 5, y = 17, w = 5, h = 17},
		{id = "gauge-y2", src = 3, x = 10, y = 17, w = 5, h = 17},
		{id = "gauge-p2", src = 3, x = 15, y = 17, w = 5, h = 17},
		{id = "gauge-y3", src = 3, x = 10, y = 34, w = 5, h = 17},
		{id = "gauge-p3", src = 3, x = 15, y = 34, w = 5, h = 17},

		-- Judge word sprites.
		{id = "judgef-pg", src = 4, x = 0, y = 0, w = 180, h = 100, divy = 2, cycle = 100},
		{id = "judgef-gr", src = 4, x = 0, y = 150, w = 180, h = 50},
		{id = "judgef-gd", src = 4, x = 0, y = 200, w = 180, h = 50},
		{id = "judgef-bd", src = 4, x = 0, y = 250, w = 180, h = 50},
		{id = "judgef-pr", src = 4, x = 0, y = 300, w = 180, h = 50},
		{id = "judgef-ms", src = 4, x = 0, y = 300, w = 180, h = 50},

		-- Early/Late indicator.
		{id = "judge-early", src = 13, x = 0, y = 0, w = 50, h = 20},
		{id = "judge-late", src = 13, x = 50, y = 0, w = 50, h = 20}
	}

	-- Bomb / hold effect animations for each key.
	local function make_bomb(idx, prefix, sy, timer_fn)
		local label = idx
		if idx == 25 then label = "su"
		elseif idx == 26 then label = "sd" end
		return {id = prefix .. label, src = 10, x = 0, y = sy, w = 1810, h = 192, divx = 10, timer = timer_fn(idx), cycle = 160}
	end
	for i = 1, 8 do
		table.insert(skin.image, make_bomb(i, "bomb1-", 0, bomb_timer))
		table.insert(skin.image, make_bomb(i, "bomb2-", 576, bomb_timer))
		table.insert(skin.image, make_bomb(i, "bomb3-", 192, bomb_timer))
		table.insert(skin.image, make_bomb(i, "hold-", 384, hold_timer))
	end

	-- Imagesets: key beam selection based on judge result, bomb selection.
	skin.imageset = {}
	do
		local suffixes = { "w", "b", "s" }
		for i = 1, 8 do
			local s = suffixes[key_type(i) + 1]
			table.insert(skin.imageset, {
				id = "keybeam" .. i,
				ref = judge_value(i),
				images = { "keybeam-" .. s, "keybeam-" .. s .. "-pg" }
			})
		end
	end
	for i = 1, 8 do
		table.insert(skin.imageset, {
			id = i + 109,
			ref = judge_value(i),
			images = { "bomb1-" .. i, "bomb2-" .. i, "bomb1-" .. i, "bomb3-" .. i }
		})
	end

	-- Numeric value displays.
	skin.value = {
		{id = "minbpm", src = 5, x = 0, y = 0, w = 240, h = 24, divx = 10, digit = 4, ref = 91},
		{id = "nowbpm", src = 5, x = 0, y = 0, w = 240, h = 24, divx = 10, digit = 4, ref = 160},
		{id = "maxbpm", src = 5, x = 0, y = 0, w = 240, h = 24, divx = 10, digit = 4, ref = 90},
		{id = "timeleft-m", src = 5, x = 0, y = 0, w = 240, h = 24, divx = 10, digit = 2, ref = 163},
		{id = "timeleft-s", src = 5, x = 0, y = 0, w = 240, h = 24, divx = 10, digit = 2, padding = 1, ref = 164},
		{id = "hispeed", src = 5, x = 0, y = 0, w = 240, h = 24, divx = 10, digit = 2, ref = 310},
		{id = "hispeed-d", src = 5, x = 0, y = 0, w = 240, h = 24, divx = 10, digit = 2, padding = 1, ref = 311},
		{id = "duration", src = 5, x = 0, y = 0, w = 240, h = 24, divx = 10, digit = 4, ref = 312},
		{id = "gaugevalue", src = 5, x = 0, y = 0, w = 240, h = 24, divx = 10, digit = 3, ref = 107},
		{id = "gaugevalue-d", src = 5, x = 0, y = 0, w = 240, h = 24, divx = 10, digit = 1, ref = 407},
		{id = "graphvalue-rate", src = 5, x = 0, y = 0, w = 240, h = 24, divx = 10, digit = 4, ref = 102},
		{id = "graphvalue-rate-d", src = 5, x = 0, y = 0, w = 240, h = 24, divx = 10, digit = 1, ref = 103},
		{id = 422, src = 5, x = 0, y = 0, w = 240, h = 24, divx = 10, digit = 5, ref = 71},
		{id = 423, src = 5, x = 0, y = 24, w = 240, h = 24, divx = 10, digit = 5, ref = 150},
		{id = 424, src = 5, x = 0, y = 48, w = 240, h = 24, divx = 10, digit = 5, ref = 121},

		{id = "lanecover-value", src = 0, x = 0, y = 550, w = 100, h = 15, divx = 10, digit = 3, ref = 14},
		{id = "lanecover-duration", src = 0, x = 0, y = 565, w = 100, h = 15, divx = 10, digit = 4, ref = 312},

		-- Judge combo numbers.
		{id = "judgen-pg", src = 4, x = 200, y = 0, w = 300, h = 100, divx = 10, divy = 2, digit = 6, ref = 75, cycle = 100},
		{id = "judgen-gr", src = 4, x = 200, y = 150, w = 300, h = 50, divx = 10, digit = 6, ref = 75},
		{id = "judgen-gd", src = 4, x = 200, y = 200, w = 300, h = 50, divx = 10, digit = 6, ref = 75},
		{id = "judgen-bd", src = 4, x = 200, y = 250, w = 300, h = 50, divx = 10, digit = 6, ref = 75},
		{id = "judgen-pr", src = 4, x = 200, y = 300, w = 300, h = 50, divx = 10, digit = 6, ref = 75},
		{id = "judgen-ms", src = 4, x = 200, y = 300, w = 300, h = 50, divx = 10, digit = 6, ref = 75},

		-- Timing ms display.
		{id = "judgems-1pp", src = 13, x = 0, y = 20, w = 120, h = 40, divx = 12, divy = 2, digit = 4, ref = 525},
		{id = "judgems-1pg", src = 13, x = 0, y = 60, w = 120, h = 40, divx = 12, divy = 2, digit = 4, ref = 525}
	}
	extend(skin.value, parts.judge_count_sources("judge-count-", 5))

	skin.text = {
		{id = "song-title", font = 0, size = 24, align = geo.title_align, ref = 12}
	}

	skin.slider = {
		{id = "musicprogress", src = 0, x = 0, y = 289, w = 14, h = 20, angle = 2, range = geo.prog_h - 20, type = 6},
		{id = "musicprogress-fin", src = 0, x = 15, y = 289, w = 14, h = 20, angle = 2, range = geo.prog_h - 20, type = 6},
		{id = "lanecover", src = 12, x = 0, y = 0, w = 390, h = 580, angle = 2, range = 580, type = 4}
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

	-- Note field definition.
	local wbs7 = {"note-w","note-b","note-w","note-b","note-w","note-b","note-w","note-s"}
	local function note_array(prefix)
		local t = {}
		for i = 1, 7 do
			t[i] = prefix .. (i % 2 == 1 and "w" or "b")
		end
		t[8] = prefix .. "s"
		return t
	end
	skin.note = {
		id = "notes",
		note = wbs7,
		lnstart = note_array("lns-"),
		lnend = note_array("lne-"),
		lnbody = note_array("lnb-"),
		lnactive = note_array("lna-"),
		hcnstart = note_array("hcns-"),
		hcnend = note_array("hcne-"),
		hcnbody = note_array("hcnb-"),
		hcnactive = note_array("hcna-"),
		hcndamage = note_array("hcnd-"),
		hcnreactive = note_array("hcnr-"),
		mine = note_array("mine-"),
		hidden = {},
		processed = {},
		group = {
			{id = "section-line", offset = 3, dst = {
				{x = geo.lanes_x, y = 140, w = geo.lanes_w, h = 1, r = 128, g = 128, b = 128}
			}}
		},
		time = {
			{id = "section-line", offset = 3, dst = {
				{x = geo.lanes_x, y = 140, w = geo.lanes_w, h = 1, r = 64, g = 192, b = 192}
			}}
		},
		bpm = {
			{id = "section-line", offset = 3, dst = {
				{x = geo.lanes_x, y = 140, w = geo.lanes_w, h = 2, r = 0, g = 192, b = 0}
			}}
		},
		stop = {
			{id = "section-line", offset = 3, dst = {
				{x = geo.lanes_x, y = 140, w = geo.lanes_w, h = 2, r = 192, g = 192, b = 0}
			}}
		}
	}
	skin.note.dst = {}
	for i = 1, 8 do
		skin.note.dst[i] = {
			x = geo.nx[i], y = 140, w = geo.nwid[i], h = 580
		}
	end

	-- Gauge layout (6 gauge types x 6 states).
	skin.gauge = {
		id = "gauge",
		nodes = {
			"gauge-r1","gauge-p1","gauge-r2","gauge-p2","gauge-r3","gauge-p3",
			"gauge-r1","gauge-p1","gauge-r2","gauge-p2","gauge-r3","gauge-p3",
			"gauge-r1","gauge-b1","gauge-r2","gauge-b2","gauge-r3","gauge-b3",
			"gauge-r1","gauge-p1","gauge-r2","gauge-p2","gauge-r3","gauge-p3",
			"gauge-y1","gauge-p1","gauge-y2","gauge-p2","gauge-y3","gauge-p3",
			"gauge-p1","gauge-p1","gauge-p2","gauge-p2","gauge-p3","gauge-p3"
		}
	}

	-- Judge display.
	skin.judge = {
		{
			id = "judge",
			index = 0,
			images = {
				{id = "judgef-pg", loop = -1, timer = 46, offsets = {3, 32}, dst = {
					{time = 0, x = geo.judge_x, y = 240, w = 180, h = 40},
					{time = 500}
				}},
				{id = "judgef-gr", loop = -1, timer = 46, offsets = {3, 32}, dst = {
					{time = 0, x = geo.judge_x, y = 240, w = 180, h = 40},
					{time = 500}
				}},
				{id = "judgef-gd", loop = -1, timer = 46, offsets = {3, 32}, dst = {
					{time = 0, x = geo.judge_x, y = 240, w = 180, h = 40},
					{time = 500}
				}},
				{id = "judgef-bd", loop = -1, timer = 46, offsets = {3, 32}, dst = {
					{time = 0, x = geo.judge_x, y = 240, w = 180, h = 40},
					{time = 500}
				}},
				{id = "judgef-pr", loop = -1, timer = 46, offsets = {3, 32}, dst = {
					{time = 0, x = geo.judge_x, y = 240, w = 180, h = 40},
					{time = 500}
				}},
				{id = "judgef-ms", loop = -1, timer = 46, offsets = {3, 32}, dst = {
					{time = 0, x = geo.judge_x, y = 240, w = 180, h = 40},
					{time = 500}
				}}
			},
			numbers = {
				{id = "judgen-pg", loop = -1, timer = 46, offsets = {3, 32}, dst = {
					{time = 0, x = 200, y = 0, w = 40, h = 40},
					{time = 500}
				}},
				{id = "judgen-gr", loop = -1, timer = 46, offsets = {3, 32}, dst = {
					{time = 0, x = 200, y = 0, w = 40, h = 40},
					{time = 500}
				}},
				{id = "judgen-gd", loop = -1, timer = 46, offsets = {3, 32}, dst = {
					{time = 0, x = 200, y = 0, w = 40, h = 40},
					{time = 500}
				}},
				{id = "judgen-bd", loop = -1, timer = 46, offsets = {3, 32}, dst = {
					{time = 0, x = 200, y = 0, w = 40, h = 40},
					{time = 500}
				}},
				{id = "judgen-pr", loop = -1, timer = 46, offsets = {3, 32}, dst = {
					{time = 0, x = 200, y = 0, w = 40, h = 40},
					{time = 500}
				}},
				{id = "judgen-ms", loop = -1, timer = 46, offsets = {3, 32}, dst = {
					{time = 0, x = 200, y = 0, w = 40, h = 40},
					{time = 500}
				}}
			},
			shift = true
		}
	}

	skin.bga = { id = "bga" }
	skin.judgegraph = { {id = "judgegraph", type = 1, backTexOff = 1} }
	skin.bpmgraph = { {id = "bpmgraph"} }
	skin.timingvisualizer = { {id = "timing"} }

	-- Destination layout.
	skin.destination = {
		{id = "background", dst = {
			{x = 0, y = 0, w = 1280, h = 720}
		}},
		{id = 1, dst = {
			{x = 0, y = 0, w = 1280, h = 720}
		}},
		{id = "minbpm", dst = {
			{x = 520, y = 2, w = 18, h = 18}
		}},
		{id = "nowbpm", dst = {
			{x = 592, y = 2, w = 24, h = 24}
		}},
		{id = "maxbpm", dst = {
			{x = 688, y = 2, w = 18, h = 18}
		}},
		{id = "timeleft-m", dst = {
			{x = 1148, y = 2, w = 24, h = 24}
		}},
		{id = "timeleft-s", dst = {
			{x = 1220, y = 2, w = 24, h = 24}
		}},
		{id = "hispeed", dst = {
			{x = 116, y = 2, w = 12, h = 24}
		}},
		{id = "hispeed-d", dst = {
			{x = 154, y = 2, w = 10, h = 20}
		}},
		{id = "duration", dst = {
			{x = 318, y = 2, w = 12, h = 24}
		}},
		{id = 13, dst = {
			{x = geo.prog_x + 2, y = geo.prog_y, w = geo.prog_w - 4, h = geo.prog_h}
		}},
		{id = "lane-bg", loop = 1000, offset = 44, dst = {
			{time = 0, x = geo.lanebg_x, y = 140, w = geo.lanebg_w, h = 0, a = 0},
			{time = 1000, h = 580, a = 255}
		}},
		{id = "keys", dst = {
			{x = geo.lanes_x, y = 100, w = geo.lanes_w, h = 80}
		}}
	}

	-- Key beams.
	for _, i in ipairs(beam_order) do
		table.insert(skin.destination, {
			id = "keybeam" .. i,
			timer = keyon_timer(i),
			loop = 100,
			offsets = {3, 40},
			dst = {
				{time = 0, x = geo.nx[i] + geo.nwid[i] / 4, y = 140, w = geo.nwid[i] / 2, h = 580},
				{time = 100, x = geo.nx[i], w = geo.nwid[i]}
			}
		})
	end

	-- Judge line.
	table.insert(skin.destination, {id = 15, offset = 50, dst = {{x = geo.lanes_x, y = 137, w = geo.lanes_w, h = 6}}})

	-- Notes field.
	table.insert(skin.destination, {id = "notes", offset = 30})

	-- Bomb effects.
	for i = 1, 8 do
		table.insert(skin.destination, {
			id = 109 + i,
			timer = bomb_timer(i),
			blend = 2,
			loop = -1,
			offsets = {3, 41},
			dst = {
				{time = 0, x = geo.center[i] - 80, y = 28, w = 180, h = 192},
				{time = 160}
			}
		})
	end

	-- Hold effects.
	for i = 1, 8 do
		table.insert(skin.destination, {
			id = "hold-" .. i,
			timer = hold_timer(i),
			blend = 2,
			offset = 3,
			dst = {
				{time = 0, x = geo.center[i] - 80, y = 28, w = 180, h = 192}
			}
		})
	end

	-- Judge display, early/late indicators, hidden/lane covers, gauge, BGA, graphs.
	extend(skin.destination, {
		{id = "judge"},
		{id = "judge-early", loop = -1, timer = 46, op = {911, 1242}, offsets = {3, 33}, dst = {
			{time = 0, x = geo.jdetail_x, y = geo.jdetail_y, w = 50, h = 20},
			{time = 500}
		}},
		{id = "judge-late", loop = -1, timer = 46, op = {911, 1243}, offsets = {3, 33}, dst = {
			{time = 0, x = geo.jdetail_x, y = geo.jdetail_y, w = 50, h = 20},
			{time = 500}
		}},
		{id = "judgems-1pp", loop = -1, timer = 46, op = {912, 241}, offsets = {3, 33}, dst = {
			{time = 0, x = geo.jdetail_x, y = geo.jdetail_y, w = 10, h = 20},
			{time = 500}
		}},
		{id = "judgems-1pg", loop = -1, timer = 46, op = {912, -241}, offsets = {3, 33}, dst = {
			{time = 0, x = geo.jdetail_x, y = geo.jdetail_y, w = 10, h = 20},
			{time = 500}
		}},
		{id = "hidden-cover", dst = {
			{x = geo.lanes_x, y = -440, w = geo.lanes_w, h = 580}
		}},
		{id = "lanecover", dst = {
			{x = geo.lanes_x, y = 720, w = geo.lanes_w, h = 580}
		}},
		{id = "gauge", dst = {
			{time = 0, x = geo.gauge_x, y = 30, w = geo.gauge_w, h = 30}
		}},
		{id = "gaugevalue", dst = {
			{time = 0, x = geo.gval_x, y = 60, w = 24, h = 24}
		}},
		{id = "gaugevalue-d", dst = {
			{time = 0, x = geo.gval_x + 72, y = 60, w = 18, h = 18}
		}}
	})

	extend(skin.destination, {
		{id = "bga", offset = 43, dst = {
			{time = 0, x = geo.bga_x, y = geo.bga_y, w = geo.bga_w, h = geo.bga_h}
		}},
		{id = "judgegraph", dst = {
			{time = 0, x = geo.jgraph_x, y = geo.jgraph_y, w = geo.jgraph_w, h = geo.jgraph_h}
		}},
		{id = "bpmgraph", dst = {
			{time = 0, x = geo.jgraph_x, y = geo.jgraph_y, w = geo.jgraph_w, h = geo.jgraph_h}
		}},
		{id = "timing", dst = {
			{time = 0, x = geo.timing_x, y = geo.timing_y, w = geo.timing_w, h = geo.timing_h}
		}},
		{id = "song-title", dst = {
			{time = 0, x = geo.title_x, y = 674, w = 24, h = 24},
			{time = 1000, a = 0},
			{time = 2000, a = 255}
		}},
		{id = 11, op = {901}, dst = {
			{x = geo.graph_x, y = geo.graph_y, w = geo.graph_w, h = geo.graph_h}
		}},
		{id = "graph-now", op = {901}, dst = {
			{x = geo.graph_x + 1, y = geo.graph_y, w = geo.graph_w / 3 - 2, h = geo.graph_h}
		}},
		{id = "graph-best", op = {901}, dst = {
			{x = geo.graph_x + geo.graph_w / 3 + 1, y = geo.graph_y, w = geo.graph_w / 3 - 2, h = geo.graph_h}
		}},
		{id = "graph-target", op = {901}, dst = {
			{x = geo.graph_x + geo.graph_w * 2 / 3 + 1, y = geo.graph_y, w = geo.graph_w / 3 - 2, h = geo.graph_h}
		}},
		{id = 12, op = {901}, dst = {
			{x = geo.graph_x, y = geo.graph_y, w = geo.graph_w, h = geo.graph_h}
		}},
		{id = "graphvalue-rate", op = {901}, dst = {
			{x = geo.graph_x + 10, y = 200, w = 12, h = 18}
		}},
		{id = "graphvalue-rate-d", op = {901}, dst = {
			{x = geo.graph_x + 58, y = 200, w = 8, h = 12}
		}},
		{id = 422, op = {901}, dst = {
			{x = geo.graph_x + 10, y = 180, w = 12, h = 18}
		}},
		{id = 423, op = {901}, dst = {
			{x = geo.graph_x + 10, y = 160, w = 12, h = 18}
		}},
		{id = 424, op = {901}, dst = {
			{x = geo.graph_x + 10, y = 140, w = 12, h = 18}
		}},
		{id = "musicprogress", blend = 2, dst = {
			{x = geo.prog_x, y = geo.prog_y + geo.prog_h - 20, w = geo.prog_w, h = 20}
		}},
		{id = "musicprogress-fin", blend = 2, timer = 143, dst = {
			{x = geo.prog_x, y = geo.prog_y + geo.prog_h - 20, w = geo.prog_w, h = 20}
		}},
	})

	-- Judge count overlays.
	extend(skin.destination, parts.judge_count_destinations("judge-count-", geo.jcount_x, geo.jcount_y, {906}, 42))

	-- Lane cover value display, load progress, ready, close, and fade.
	extend(skin.destination, {
		{id = "lanecover-value", offset = 4, op = {270}, dst = {
			{time = 0, x = geo.lanes_x + geo.lanes_w / 3 - 24, y = 720, w = 12, h = 18}
		}},
		{id = "lanecover-duration", offset = 4, op = {270}, dst = {
			{time = 0, x = geo.lanes_x + geo.lanes_w * 2 / 3 - 24, y = 720, w = 12, h = 18}
		}},
		{id = "load-progress", loop = 0, op = {80}, dst = {
			{time = 0, x = geo.lanes_x, y = 440, w = geo.lanes_w, h = 4},
			{time = 500, a = 192, r = 0},
			{time = 1000, a = 128, r = 255, g = 0},
			{time = 1500, a = 192, g = 255, b = 0},
			{time = 2000, a = 255, b = 255}
		}},
		{id = "ready", loop = -1, timer = 40, dst = {
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
