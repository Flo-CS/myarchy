hl.config({
	general = {
		col = {
			active_border = "rgb(56685d)",
			inactive_border = "rgba(1e2d24ff)",
		},
	},

	group = {
		col = {
			border_active = "rgb(56685d)",
			border_inactive = "rgba(1e2d24ff)",
		},
		groupbar = {
			col = {
				active = "rgba(111a14cc)",
				inactive = "rgba(060906aa)",
			},
			text_color = "rgba(c9d5cbff)",
			text_color_inactive = "rgba(56685dff)",
		},
	},
})

hl.window_rule({
	name = "theme-single-tiled-border",
	match = { float = false, workspace = "w[tv1]" },
	border_color = "rgba(1e2d24ff)",
})

hl.window_rule({
	name = "theme-single-fullscreen-border",
	match = { float = false, workspace = "f[1]" },
	border_color = "rgba(1e2d24ff)",
})
