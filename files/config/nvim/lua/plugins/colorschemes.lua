return {
  {
    'catppuccin/nvim',
    name = 'catppuccin',
    priority = 1000,
    lazy = false,
    config = function()
      require('catppuccin').setup {
        transparent_background = true,
      }
    end,
  },
  {
    'folke/tokyonight.nvim',
    priority = 1000,
    lazy = false,
    config = function()
      require('tokyonight').setup {
        transparent = true,
      }
    end,
  },
  {
    'rose-pine/neovim',
    name = 'rose-pine',
    priority = 1000,
    lazy = false,
    config = function()
      require('rose-pine').setup {
        transparent = true,
      }
    end,
  },
  {
    'ribru17/bamboo.nvim',
    lazy = false,
    priority = 1000,
    lazy = false,
    config = function()
      require('bamboo').setup {
        transparent = true,
      }
    end,
  },
}
