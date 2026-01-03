return {
  {
    'folke/trouble.nvim',
    opts = {},
    cmd = 'Trouble',
    keys = {
      {
        '<leader>dx',
        '<cmd>Trouble diagnostics toggle<cr>',
        desc = '[d]iagnostics [x] all',
      },
      {
        '<leader>dX',
        '<cmd>Trouble diagnostics toggle filter.buf=0<cr>',
        desc = '[d]iagnostics [X] buffer',
      },
    },
  },
}
