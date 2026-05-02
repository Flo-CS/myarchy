return {
  {
    'stevearc/oil.nvim',
    lazy = false,
    dependencies = { 'nvim-tree/nvim-web-devicons' },
    keys = {
      { '<leader>e', ':Oil --float<CR>', desc = 'Explorer' },
    },
    opts = {
      view_options = {
        show_hidden = true,
      },
      keymaps = {
        ['<esc>'] = { 'actions.close', mode = 'n' },
        ['q'] = { 'actions.close', mode = 'n' },
      },
      float = {
        max_width = 0.8,
        max_height = 0.8,
        border = 'rounded',
      },
    },
  },
}
