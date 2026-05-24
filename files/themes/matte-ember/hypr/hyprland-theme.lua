hl.config({
	general = {
		col = {
			active_border = "rgb(7a7c80)",
			inactive_border = "rgba(26282dff)",
		},
	},

	group = {
		col = {
			border_active = "rgb(7a7c80)",
			border_inactive = "rgba(26282dff)",
		},
		groupbar = {
			col = {
				active = "rgba(121214cc)",
				inactive = "rgba(050506aa)",
			},
			text_color = "rgba(d7d8dcff)",
			text_color_inactive = "rgba(7a7c80ff)",
		},
	},
})

hl.window_rule({
	name = "theme-single-tiled-border",
	match = { float = false, workspace = "w[tv1]" },
	border_color = "rgba(26282dff)",
})

hl.window_rule({
	name = "theme-single-fullscreen-border",
	match = { float = false, workspace = "f[1]" },
	border_color = "rgba(26282dff)",
})
