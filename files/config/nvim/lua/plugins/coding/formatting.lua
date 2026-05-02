return {
  {
    'stevearc/conform.nvim',
    dependencies = { 'mason-org/mason.nvim' },
    event = { 'VeryLazy' },
    keys = {
      {
        '<leader>cf',
        function()
          require('conform').format()
        end,
        mode = { 'n', 'x' },
        desc = 'Format',
      },
    },
    opts = {
      default_format_opts = {
        timeout_ms = 1000,
        async = false,
        quiet = false,
        lsp_format = 'fallback',
      },
      format_on_save = {},
    },
    formatters_by_ft = {},
  },
}
