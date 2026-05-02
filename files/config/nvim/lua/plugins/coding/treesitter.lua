return {
  {
    'nvim-treesitter/nvim-treesitter',
    lazy = false,
    build = ':TSUpdate',
    dependencies = {
      'nvim-treesitter/nvim-treesitter-textobjects',
    },
    opts = {
      ensure_installed = {},
      auto_install = true,
      folds = { enable = true },
      highlight = { enable = true },
      indent = { enable = true },
      incremental_selection = {
        enable = true,
        keymaps = {
          init_selection = '<C-space>',
          node_incremental = '<C-space>',
          scope_incremental = false,
          node_decremental = '<bs>',
        },
      },
    },
  },
  {
    'nvim-treesitter/nvim-treesitter-textobjects',
    lazy = false,
    opts = {
      select = {
        lookahead = true,
      },
      move = {
        set_jumps = true,
      },
    },
    config = function(_, opts)
      require('nvim-treesitter-textobjects').setup(opts)

      local select = require('nvim-treesitter-textobjects.select')
      local move = require('nvim-treesitter-textobjects.move')

      -- select
      local selections = {
        ['ak'] = '@block.outer',
        ['ik'] = '@block.inner',
        ['ac'] = '@class.outer',
        ['ic'] = '@class.inner',
        ['a?'] = '@conditional.outer',
        ['i?'] = '@conditional.inner',
        ['af'] = '@function.outer',
        ['if'] = '@function.inner',
        ['ao'] = '@loop.outer',
        ['io'] = '@loop.inner',
        ['aa'] = '@parameter.outer',
        ['ia'] = '@parameter.inner',
      }
      for lhs, query in pairs(selections) do
        vim.keymap.set({ 'x', 'o' }, lhs, function()
          select.select_textobject(query, 'textobjects')
        end, { desc = 'Select ' .. query })
      end

      -- move
      local moves = {
        goto_next_start = { [']k'] = '@block.outer', [']f'] = '@function.outer', [']a'] = '@parameter.inner' },
        goto_next_end = { [']K'] = '@block.outer', [']F'] = '@function.outer', [']A'] = '@parameter.inner' },
        goto_previous_start = { ['[k'] = '@block.outer', ['[f'] = '@function.outer', ['[a'] = '@parameter.inner' },
        goto_previous_end = { ['[K'] = '@block.outer', ['[F'] = '@function.outer', ['[A'] = '@parameter.inner' },
      }
      for method, maps in pairs(moves) do
        for lhs, query in pairs(maps) do
          vim.keymap.set({ 'n', 'x', 'o' }, lhs, function()
            move[method](query, 'textobjects')
          end, { desc = method .. ' ' .. query })
        end
      end
    end,
  },
}
