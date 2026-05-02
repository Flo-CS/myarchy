return {
  {
    'mfussenegger/nvim-lint',
    dependencies = { 'mason-org/mason.nvim' },
    event = { 'VeryLazy' },
    opts = {
      events = { 'BufWritePost', 'BufReadPost', 'InsertLeave' },
      linters_by_ft = {},
    },
    config = function(_, opts)
      local lint = require('lint')

      lint.linters_by_ft = opts.linters_by_ft
      vim.api.nvim_create_autocmd(opts.events, {
        callback = function()
          lint.try_lint()
        end,
      })
    end,
  },
}
