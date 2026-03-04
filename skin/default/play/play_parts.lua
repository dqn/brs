-- Shared helpers for judge count value sources and destination layouts.
-- Used by play7main and play24main via require("play_parts").

local judge_names = { "pf", "gr", "gd", "bd", "pr", "ms" }
local timing_variants = { "", "-e", "-l" }
local early_late_only = { "-e", "-l" }

-- Map (judge_index, timing_index) to the engine value reference id.
local function judge_count_ref(ji, ti)
	if ji <= 5 then
		if ti == 1 then
			return 109 + ji
		else
			return 410 + (ji - 1) * 2 + (ti - 2)
		end
	else
		return 420 + (ti - 1)
	end
end

return {
	-- Build value source entries for judge counts (total / early / late per rank).
	judge_count_sources = function(id_prefix, num_src_id)
		local out = {}
		for ji, jname in ipairs(judge_names) do
			for ti, tsuffix in ipairs(timing_variants) do
				table.insert(out, {
					id = id_prefix .. jname .. tsuffix,
					src = num_src_id,
					x = 0,
					y = (ti - 1) * 24,
					w = 264,
					h = 24,
					divx = 11,
					digit = 4,
					ref = judge_count_ref(ji, ti),
				})
			end
		end
		return out
	end,

	-- Build destination entries for early/late judge count display.
	judge_count_destinations = function(id_prefix, base_x, base_y, ops, ofs)
		local out = {}
		for row, jname in ipairs(judge_names) do
			for col, tsuffix in ipairs(early_late_only) do
				local entry = {
					id = id_prefix .. jname .. tsuffix,
					op = ops,
					dst = {
						{x = base_x + (col - 1) * 60, y = base_y + (row - 1) * 18, w = 12, h = 18}
					}
				}
				if ofs >= 0 then
					entry.offset = ofs
				end
				table.insert(out, entry)
			end
		end
		return out
	end
}
