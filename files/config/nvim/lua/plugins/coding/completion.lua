return {
  {
    'windwp/nvim-autopairs',
    event = 'VeryLazy',
    opts = {},
  },
  {
    'windwp/nvim-ts-autotag',
    event = 'VeryLazy',
    opts = {},
  },
  {
    'folke/ts-comments.nvim',
    event = 'VeryLazy',
    opts = {},
  },
  {
    'kylechui/nvim-surround',
    version = '^4.0.0',
    event = 'VeryLazy',
    opts = {},
  },
  {
    'saghen/blink.cmp',
    version = '1.*',
    dependencies = { 'rafamadriz/friendly-snippets' },
    event = 'VeryLazy',
    --- @module 'blink.cmp'
    --- @type blink.cmp.Config
    opts = {
      snippets = {
        preset = 'default',
      },
      keymap = {
        preset = 'super-tab',
        ['<CR>'] = {
          function(cmp)
            if cmp.snippet_active() then
              return cmp.accept()
            else
              return cmp.select_and_accept()
            end
          end,
          'snippet_forward',
          'fallback',
        },
      },
      appearance = {
        nerd_font_variant = 'mono',
        use_nvim_cmp_as_default = false,
      },
      completion = {
        list = {
          selection = {
            preselect = true,
            auto_insert = false,
          },
        },
        documentation = {
          auto_show = true,
          auto_show_delay_ms = 500,
          window = { border = 'rounded' },
          treesitter_highlighting = true,
        },
        menu = {
          auto_show = true,
          auto_show_delay_ms = 0,
          border = 'rounded',
          draw = {
            treesitter = { 'lsp' },
          },
        },
        ghost_text = { enabled = false },
        accept = { auto_brackets = { enabled = true } },
      },
      signature = {
        enabled = true,
        window = { border = 'rounded' },
      },
      sources = {
        default = { 'lazydev', 'lsp', 'path', 'snippets', 'buffer' },
        providers = {
          lazydev = {
            name = 'LazyDev',
            module = 'lazydev.integrations.blink',
            score_offset = 100,
          },
        },
      },
      cmdline = {
        enabled = true,
        keymap = { preset = 'cmdline' },
        completion = {
          menu = { auto_show = true },
          ghost_text = { enabled = true },
        },
      },
      fuzzy = {
        implementation = 'prefer_rust_with_warning',
      },
    },
    opts_extend = { 'sources.default' },
  },
}
