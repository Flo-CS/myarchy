return {
  {
    'folke/noice.nvim',
    event = 'VeryLazy',
    dependencies = {
      'MunifTanjim/nui.nvim',
    },
    opts = {
      lsp = {
        override = {
          ['vim.lsp.util.convert_input_to_markdown_lines'] = true,
          ['vim.lsp.util.stylize_markdown'] = true,
        },
      },
      presets = {
        bottom_search = false,
        command_palette = true,
        long_message_to_split = true,
        inc_rename = true,
        lsp_doc_border = true,
      },
    },
  },
  {
    'folke/snacks.nvim',
    priority = 1000,
    --- @module 'snacks'
    --- @type snacks.Config
    opts = {
      words = {
        enabled = true,
        debounce = 50,
      },
      indent = {
        enabled = true,
        char = '┆',
        chunk = {
          enabled = true,
          char = {
            horizontal = '─',
            vertical = '│',
            corner_top = '╭',
            corner_bottom = '╰',
            arrow = '─',
          },
        },
        animate = {
          enabled = true,
          duration = {
            total = 100,
          },
        },
      },
    },
  },
  {
    'sphamba/smear-cursor.nvim',
    event = 'VeryLazy',
    opts = {
      stiffness = 0.8,
      trailing_stiffness = 0.6,
      stiffness_insert_mode = 0.7,
      trailing_stiffness_insert_mode = 0.7,
      damping = 0.95,
      damping_insert_mode = 0.95,
      distance_stop_animating = 0.5,
      time_interval = 7,
    },
  },
  {
    'catgoose/nvim-colorizer.lua',
    event = 'BufReadPre',
    --- @module 'colorizer'
    --- @type colorizer.Options
    opts = {
      options = {
        parsers = {
          hex = { default = true },
          rgb = { enable = true },
          hsl = { enable = true },
        },
      },
    },
  },
}
