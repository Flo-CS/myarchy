return {
  {
    'folke/trouble.nvim',
    event = { 'VeryLazy' },
    keys = {
      { '<leader>xx', '<cmd>Trouble diagnostics toggle<cr>', desc = 'Diagnostics' },
      { '<leader>xX', '<cmd>Trouble diagnostics toggle filter.buf=0<cr>', desc = 'Diagnostics Buffer' },
      { '<leader>xl', '<cmd>Trouble loclist toggle<cr>', desc = 'Locs' },
      { '<leader>xq', '<cmd>Trouble qflist toggle<cr>', desc = 'Quickfixs' },
      { '<leader>cs', '<cmd>Trouble symbols toggle<cr>', desc = 'LSP Symbols' },
      { '<leader>cS', '<cmd>Trouble lsp toggle<cr>', desc = 'LSP References, definitions...' },
    },
    opts = {
      open_no_results = true,
      modes = {
        lsp = { win = { position = 'right' } },
        qflist = { auto_close = true, focus = true },
        loclist = { auto_close = true, focus = true },
      },
    },
    config = function(_, opts)
      require('trouble').setup(opts)

      vim.api.nvim_create_autocmd('BufRead', {
        callback = function(ev)
          if vim.bo[ev.buf].buftype == 'quickfix' then
            vim.schedule(function()
              vim.cmd([[cclose]])
              vim.cmd([[Trouble qflist open]])
            end)
          end
          if vim.bo[ev.buf].buftype == 'loclist' then
            vim.schedule(function()
              vim.cmd([[lclose]])
              vim.cmd([[Trouble loclist open]])
            end)
          end
        end,
      })
    end,
  },
  {
    'folke/todo-comments.nvim',
    dependencies = { 'nvim-lua/plenary.nvim' },
    event = { 'VeryLazy' },
    keys = {
      { '<leader>xt', '<cmd>Trouble todo toggle<cr>', desc = 'TODOs' },
      { '<leader>st', '<cmd>TodoTelescope<cr>', desc = 'TODOs' },
    },
    opts = {},
  },
}
