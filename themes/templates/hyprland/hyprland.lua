hl.config({
	general = {
		col = {
			active_border = "rgb({{outline-strong|raw}})",
			inactive_border = "rgba({{outline|raw:ff}})",
		},
	},

	group = {
		col = {
			border_active = "rgb({{outline-strong|raw}})",
			border_inactive = "rgba({{outline|raw:ff}})",
		},
		groupbar = {
			col = {
				active = "rgba({{surface-2|raw:cc}})",
				inactive = "rgba({{surface|raw:aa}})",
			},
			text_color = "rgba({{foreground|raw:ff}})",
			text_color_inactive = "rgba({{foreground-muted|raw:ff}})",
		},
	},
})
