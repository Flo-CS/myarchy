return {
  {
    'mason-org/mason.nvim',
    event = { 'VeryLazy' },
    opts = {
      ui = { border = 'rounded' },
    },
  },
  {
    'mason-org/mason-lspconfig.nvim',
    dependencies = {
      'mason-org/mason.nvim',
      'neovim/nvim-lspconfig',
    },
    event = { 'VeryLazy' },
    opts = {
      ensure_installed = {},
      automatic_enable = true,
    },
  },
  {
    'neovim/nvim-lspconfig',
    event = { 'VeryLazy' },
    config = function()
      vim.api.nvim_create_autocmd('LspAttach', {
        callback = function(args)
          local map = function(keys, func, desc)
            vim.keymap.set({ 'n', 'v' }, keys, func, { buffer = args.buf, desc = desc })
          end
          map('<leader>gd', vim.lsp.buf.definition, 'Go to Definition')
          map('<leader>gi', vim.lsp.buf.implementation, 'Go to Implementation')
          map('<leader>gr', vim.lsp.buf.references, 'Go to References')
          map('<leader>ca', vim.lsp.buf.code_action, 'Action')
          map('K', vim.lsp.buf.hover, 'Hover Documentation')

          local client = vim.lsp.get_client_by_id(args.data.client_id)
          if client and client:supports_method('textDocument/inlayHint') then
            vim.lsp.inlay_hint.enable(false, { bufnr = args.buf })
          end

          vim.diagnostic.config {
            virtual_text = true,
          }

          map('<C-Space>', function()
            if vim.treesitter.get_parser(nil, nil, { error = false }) then
              require('vim.treesitter._select').select_parent(vim.v.count1)
            else
              vim.lsp.buf.selection_range(vim.v.count1)
            end
          end, 'Increment selection')
          map('<BS>', function()
            if vim.treesitter.get_parser(nil, nil, { error = false }) then
              require('vim.treesitter._select').select_child(vim.v.count1)
            else
              vim.lsp.buf.selection_range(-vim.v.count1)
            end
          end, 'Decrement selection')
        end,
      })
    end,
  },
  {
    'smjonas/inc-rename.nvim',
    opts = {},
    keys = {
      {
        '<leader>cr',
        function()
          return ':IncRename ' .. vim.fn.expand('<cword>')
        end,
        expr = true,
        desc = 'Rename Incremental',
      },
    },
  },
}
