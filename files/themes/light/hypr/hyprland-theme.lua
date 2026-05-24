hl.config({
	general = {
		col = {
			active_border = "rgba(d4cab2ff)",
			inactive_border = "rgba(8a857a00)",
		},
	},

	group = {
		col = {
			border_active = "rgba(d4cab2ff)",
			border_inactive = "rgb(8a857a)",
		},
		groupbar = {
			col = {
				active = "rgba(f2ede0cc)",
				inactive = "rgba(ffffffaa)",
			},
			text_color = "rgba(3a3530ff)",
			text_color_inactive = "rgba(8a857aff)",
		},
	},
})

hl.window_rule({
	name = "theme-single-tiled-border",
	match = { float = false, workspace = "w[tv1]" },
	border_color = "rgb(8a857a)",
})

hl.window_rule({
	name = "theme-single-fullscreen-border",
	match = { float = false, workspace = "f[1]" },
	border_color = "rgb(8a857a)",
})
