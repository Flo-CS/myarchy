return {
  {
    'stevearc/aerial.nvim',
    dependencies = {
      'nvim-treesitter/nvim-treesitter',
      'nvim-tree/nvim-web-devicons',
    },
    config = function()
      require('aerial').setup {
        autojump = true,
        close_on_select = true,
      }
      vim.keymap.set('n', '<leader>a', '<cmd>AerialOpen<CR>', { desc = '[a]erial open' })
    end,
  },
}
