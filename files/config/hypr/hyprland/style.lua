hl.config({
	general = {
		gaps_in = 5,
		gaps_out = 10,
		gaps_workspaces = 10,
		border_size = 2,
		resize_on_border = true,
		extend_border_grab_area = 30,
		snap = {
			enabled = true,
			window_gap = 10,
			monitor_gap = 5,
		},
	},

	decoration = {
		rounding = 0,
		active_opacity = 0.95,
		inactive_opacity = 0.90,
		fullscreen_opacity = 1.0,

		blur = {
			enabled = true,
			size = 8,
			passes = 1,
		},

		shadow = {
			enabled = true,
			range = 2,
			render_power = 3,
			color = "rgba(1a1a1aee)",
		},
	},

	animations = {
		enabled = true,
	},

	group = {
		groupbar = {
			render_titles = true,
			keep_upper_gap = false,
			height = 20,
			indicator_height = 3,
			indicator_gap = 0,
			gaps_in = 0,
			gaps_out = 0,
			rounding = 2,
			gradient_rounding = 4,
			blur = true,
			font_size = 10,
			gradients = true,
		},
	},
})

-- ANIMATIONS

hl.curve("linear", { type = "bezier", points = { { 0, 0 }, { 1, 1 } } })
hl.curve("md3_standard", { type = "bezier", points = { { 0.2, 0 }, { 0, 1 } } })
hl.curve("md3_decel", { type = "bezier", points = { { 0.05, 0.7 }, { 0.1, 1 } } })
hl.curve("md3_accel", { type = "bezier", points = { { 0.3, 0 }, { 0.8, 0.15 } } })
hl.curve("overshot", { type = "bezier", points = { { 0.05, 0.9 }, { 0.1, 1.1 } } })
hl.curve("crazyshot", { type = "bezier", points = { { 0.1, 1.5 }, { 0.76, 0.92 } } })
hl.curve("hyprnostretch", { type = "bezier", points = { { 0.05, 0.9 }, { 0.1, 1.0 } } })
hl.curve("menu_decel", { type = "bezier", points = { { 0.1, 1 }, { 0, 1 } } })
hl.curve("menu_accel", { type = "bezier", points = { { 0.38, 0.04 }, { 1, 0.07 } } })
hl.curve("easeInOutCirc", { type = "bezier", points = { { 0.85, 0 }, { 0.15, 1 } } })
hl.curve("easeOutCirc", { type = "bezier", points = { { 0, 0.55 }, { 0.45, 1 } } })
hl.curve("easeOutExpo", { type = "bezier", points = { { 0.16, 1 }, { 0.3, 1 } } })
hl.curve("softAcDecel", { type = "bezier", points = { { 0.26, 0.26 }, { 0.15, 1 } } })
hl.curve("md2", { type = "bezier", points = { { 0.4, 0 }, { 0.2, 1 } } })

hl.animation({ leaf = "windows", enabled = true, speed = 2, bezier = "md3_decel", style = "popin 60%" })
hl.animation({ leaf = "windowsIn", enabled = true, speed = 2, bezier = "md3_decel", style = "popin 60%" })
hl.animation({ leaf = "windowsOut", enabled = true, speed = 2, bezier = "md3_accel", style = "popin 60%" })
hl.animation({ leaf = "border", enabled = true, speed = 5, bezier = "default" })
hl.animation({ leaf = "fade", enabled = true, speed = 2, bezier = "md3_decel" })

hl.animation({ leaf = "layersIn", enabled = true, speed = 2, bezier = "menu_decel", style = "slide" })
hl.animation({ leaf = "layersOut", enabled = true, speed = 1, bezier = "menu_accel" })
hl.animation({ leaf = "fadeLayersIn", enabled = true, speed = 1, bezier = "menu_decel" })
hl.animation({ leaf = "fadeLayersOut", enabled = true, speed = 0.5, bezier = "menu_accel" })
hl.animation({ leaf = "workspaces", enabled = true, speed = 3, bezier = "menu_decel", style = "slide" })

hl.animation({ leaf = "specialWorkspace", enabled = true, speed = 2, bezier = "md3_decel", style = "slidevert" })

-- LAYER RULES

hl.layer_rule({
	name = "waybar-blur",
	match = { namespace = "waybar" },
	blur = true,
	blur_popups = true,
	ignore_alpha = 0.5,
})

hl.layer_rule({
	name = "rofi-blur",
	match = { namespace = "rofi" },
	blur = true,
	ignore_alpha = 0.2,
})

-- BASELINE WINDOW RULES

hl.window_rule({
	name = "xwayland-blur-fix",
	match = { xwayland = true },
	no_blur = true,
})

hl.window_rule({
	name = "xwayland-drag-fix",
	match = {
		class = "^$",
		title = "^$",
		xwayland = true,
		float = true,
		fullscreen = false,
		pin = false,
	},
	no_focus = true,
})

hl.window_rule({
	name = "special-workspace-opacity",
	match = { workspace = "special:scratchpad" },
	opacity = 0.90,
})
