return {
  {
    'tpope/vim-sleuth', -- Detect tabstop and shiftwidth automatically
  },
  {
    'stevearc/conform.nvim',
    event = { 'BufReadPre', 'BufNewFile' },
    keys = {
      {
        '<leader>cf',
        function()
          require('conform').format {}
        end,
        desc = '[c]ode [f]ormat buffer',
      },
    },
    opts = {
      notify_on_error = false,
      format_on_save = function(bufnr)
        local disable_filetypes = { c = true, cpp = true }
        local lsp_format_opt
        if disable_filetypes[vim.bo[bufnr].filetype] then
          lsp_format_opt = 'never'
        else
          lsp_format_opt = 'fallback'
        end
        return {
          timeout_ms = 500,
          lsp_format = lsp_format_opt,
        }
      end,
      formatters_by_ft = {
        lua = { 'stylua' },
        json = { 'prettier' },
        jsonc = { 'prettier' },
        rust = { 'rustfmt' },
        css = { 'prettier' },
        scss = { 'prettier' },
        html = { 'prettier' },
        bash = { 'shfmt' },
        sh = { 'shfmt' },
      },
    },
  },
}
