return {
  {
    'stevearc/aerial.nvim',
    dependencies = {
      'nvim-treesitter/nvim-treesitter',
      'nvim-tree/nvim-web-devicons',
    },
    keys = {
      { '<leader>co', '<cmd>AerialOpen<CR>', desc = 'Outline' },
    },
    opts = {
      autojump = true,
      close_on_select = true,
    },
  },
}
