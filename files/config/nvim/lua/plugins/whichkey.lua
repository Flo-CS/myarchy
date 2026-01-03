return {
  {
    'folke/which-key.nvim',
    event = 'VimEnter',
    keys = {
      { '<leader>w', '<cmd>WhichKey <CR>', noremap = true, silent = true, desc = '[w]hichkey' },
    },
    opts = {
      delay = 0,
      icons = {
        mappings = vim.g.have_nerd_font and {},
        keys = vim.g.have_nerd_font and {},
      },
      spec = {
        { '<leader>c', group = '[c]ode' },
        { '<leader>g', group = '[g]it' },
        { '<leader>d', group = '[d]iagnostics' },
        { '<leader>s', group = '[s]earch' },
        { '<leader>t', group = '[t]oggle' },
        { '<leader>q', group = '[q]essions' },
      },
    },
  },
}
