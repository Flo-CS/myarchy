return {
  {
    'folke/which-key.nvim',
    event = 'VeryLazy',
    keys = {
      {
        '<leader>?',
        function()
          require('which-key').show { global = false }
        end,
        desc = 'Keymap',
      },
    },
    opts = {
      preset = 'helix',
      defaults = {},
      spec = {
        {
          mode = { 'n', 'x' },
          { '<leader><tab>', group = 'Tabs' },
          { '<leader>c', group = 'Code' },
          { '<leader>d', group = 'Debug' },
          { '<leader>dp', group = 'Profiler' },
          { '<leader>f', group = 'File' },
          { '<leader>g', group = 'Goto/Git' },
          { '<leader>gh', group = 'Hunks' },
          { '<leader>q', group = 'Session' },
          { '<leader>s', group = 'Search' },
          { '<leader>u', group = 'UI' },
          { '<leader>x', group = 'Diagnostics/Lists' },
          { '[', group = 'Prev' },
          { ']', group = 'Next' },
          { 'g', group = 'Goto' },
          { 'gs', group = 'Surround' },
          { 'z', group = 'Fold' },
          {
            '<leader>b',
            group = 'Buffer',
            expand = function()
              return require('which-key.extras').expand.buf()
            end,
          },
          {
            '<leader>w',
            group = 'Windows',
            proxy = '<c-w>',
            expand = function()
              return require('which-key.extras').expand.win()
            end,
          },
        },
      },
    },
  },
}
