return {
  {
    'nvim-treesitter/nvim-treesitter',
    branch = 'main',
    lazy = false,
    build = ':TSUpdate',
    dependencies = {
      'nvim-treesitter/nvim-treesitter-textobjects',
    },
    config = function()
      require('nvim-treesitter').install {}

      vim.api.nvim_create_autocmd('FileType', {
        group = vim.api.nvim_create_augroup('treesitter.setup', {}),
        callback = function(args)
          local buf = args.buf
          local filetype = args.match

          local language = vim.treesitter.language.get_lang(filetype) or filetype
          if not vim.treesitter.language.add(language) then
            return
          end

          -- highlight
          vim.treesitter.start(buf, language)
          -- fold
          vim.wo.foldmethod = 'expr'
          vim.wo.foldexpr = 'v:lua.vim.treesitter.foldexpr()'
          -- indent
          vim.bo[buf].indentexpr = "v:lua.require'nvim-treesitter'.indentexpr()"
        end,
      })
    end,
  },
  {
    'nvim-treesitter/nvim-treesitter-textobjects',
    branch = 'main',
    lazy = false,
    opts = {
      select = { lookahead = true },
      move = { set_jumps = true },
    },
    config = function(_, opts)
      require('nvim-treesitter-textobjects').setup(opts)
      local select = require('nvim-treesitter-textobjects.select')
      local move = require('nvim-treesitter-textobjects.move')

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

      local moves = {
        goto_next_start = {
          [']k'] = '@block.outer',
          [']f'] = '@function.outer',
          [']a'] = '@parameter.inner',
        },
        goto_next_end = {
          [']K'] = '@block.outer',
          [']F'] = '@function.outer',
          [']A'] = '@parameter.inner',
        },
        goto_previous_start = {
          ['[k'] = '@block.outer',
          ['[f'] = '@function.outer',
          ['[a'] = '@parameter.inner',
        },
        goto_previous_end = {
          ['[K'] = '@block.outer',
          ['[F'] = '@function.outer',
          ['[A'] = '@parameter.inner',
        },
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
