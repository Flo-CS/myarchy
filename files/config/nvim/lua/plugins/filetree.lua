return {
  {
    'nvim-neo-tree/neo-tree.nvim',
    version = '*',
    dependencies = {
      'nvim-lua/plenary.nvim',
      'nvim-tree/nvim-web-devicons',
      'MunifTanjim/nui.nvim',
    },
    lazy = false,
    keys = {
      -- { '<leader>o', ':Neotree reveal<CR>', desc = '[oe]xplorer' },
    },
    config = function()
      require('neo-tree').setup {
        hide_root_node = true,
        retain_hidden_root_indent = true,
        close_if_last_window = false,
        source_selector = {
          winbar = true,
        },
        default_component_configs = {
          indent = {
            with_expanders = true,
            expander_collapsed = '',
            expander_expanded = '',
          },
        },
        filesystem = {
          hijack_netrw_behavior = 'disabled',
          window = {
            position = 'float',
          },
          filtered_items = {
            hide_dotfiles = false,
            hide_gitignored = false,
            hide_hidden = true, -- only works on Windows for hidden files/directories
          },
          group_empty_dirs = false,
        },
        window = {
          auto_expand_width = false,
          mappings = {
            ['<esc>'] = 'close_window',
            ['<C-c>'] = 'close_window',
          },
        },
      }
    end,
  },
  {
    'stevearc/oil.nvim',
    dependencies = { 'nvim-tree/nvim-web-devicons' },
    lazy = false,
    keys = {
      { '<leader>e', ':Oil --float<CR>', desc = '[e]xplorer' },
    },
    config = function()
      require('oil').setup {
        view_options = {
          show_hidden = true,
        },
        keymaps = {
          ['<Esc>'] = { 'actions.close', mode = 'n' },
          ['q'] = { 'actions.close', mode = 'n' },
        },
        float = {
          max_width = 0.8,
          max_height = 0.8,
          border = 'rounded',
        },
      }
    end,
  },
}
