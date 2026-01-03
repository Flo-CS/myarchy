return {
  {
    'nvim-telescope/telescope.nvim',
    event = 'VimEnter',
    branch = '0.1.x',
    dependencies = {
      'nvim-lua/plenary.nvim',
      {
        'nvim-telescope/telescope-fzf-native.nvim',
        build = 'make',
        cond = function()
          return vim.fn.executable 'make' == 1
        end,
      },
      { 'nvim-telescope/telescope-ui-select.nvim' },
      { 'nvim-tree/nvim-web-devicons', enabled = vim.g.have_nerd_font },
    },
    config = function()
      require('telescope').setup {
        defaults = {
          mappings = {
            i = {
              ['<esc>'] = require('telescope.actions').close,
            },
          },
          layout_strategy = 'vertical',
          path_display = { shorten = 5 },
        },
        pickers = {
          live_grep = {
            additional_args = function(opts)
              return { '--hidden', '-g', '!**/.git/*,node_modules/**' }
            end,
          },
          find_files = {
            find_command = {
              'rg',
              '--files',
              '--hidden',
              '-g',
              '!**/.git/*',
            },
          },
          lsp_references = {
            path_display = { 'smart' },
          },
          lsp_definitions = {
            path_display = { 'smart' },
          },
          lsp_type_definitions = {
            path_display = { 'smart' },
          },
          lsp_implementations = {
            path_display = { 'smart' },
          },
        },
        extensions = {
          ['ui-select'] = {
            require('telescope.themes').get_dropdown(),
          },
        },
      }

      pcall(require('telescope').load_extension, 'fzf')
      pcall(require('telescope').load_extension, 'ui-select')

      local builtin = require 'telescope.builtin'
      vim.keymap.set('n', '<leader>sh', builtin.help_tags, { desc = '[s]earch [h]elp' })
      vim.keymap.set('n', '<leader>sk', builtin.keymaps, { desc = '[s]earch [k]eymaps' })
      vim.keymap.set('n', '<leader>sf', builtin.find_files, { desc = '[s]earch [f]iles' })
      vim.keymap.set('n', '<leader>sw', builtin.grep_string, { desc = '[s]earch current [w]ord' })
      vim.keymap.set('n', '<leader>sg', builtin.live_grep, { desc = '[s]earch by [g]rep' })
      vim.keymap.set('n', '<leader>sd', builtin.diagnostics, { desc = '[s]earch [d]iagnostics' })
      vim.keymap.set('n', '<leader>sr', builtin.resume, { desc = '[s]earch [r]esume' })
      vim.keymap.set('n', '<leader>sc', function()
        builtin.current_buffer_fuzzy_find(require('telescope.themes').get_dropdown {
          winblend = 10,
          previewer = false,
        })
      end, { desc = '[s]earch in [c]urrent buffer' })

      vim.keymap.set('n', '<leader>s<leader>s', builtin.builtin, { desc = '[s]earch [s]elect Telescope' })
      vim.keymap.set('n', '<leader>s<leader>n', function()
        builtin.find_files { cwd = vim.fn.stdpath 'config' }
      end, { desc = '[s]earch [n]eovim files' })
    end,
  },
}
