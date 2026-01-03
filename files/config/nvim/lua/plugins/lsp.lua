local function build_jdtls_config()
  local jdtls_workspace = vim.fn.stdpath 'data' .. '/site/java/workspace-root/' .. vim.fn.fnamemodify(vim.fn.getcwd(), ':p:h:t')
  local lombok_path = vim.fn.stdpath 'config' .. '/artifacts/java/lombok.jar'

  return {
    cmd = {
      'jdtls',
      '-data',
      jdtls_workspace,
      '--jvm-arg=-javaagent:' .. lombok_path,
      '--add-modules=ALL-SYSTEM',
      '--add-opens',
      'java.base/java.util=ALL-UNNAMED',
      '--add-opens',
      'java.base/java.lang=ALL-UNNAMED',
    },
    root_dir = require('lspconfig.util').root_pattern('.git', 'mvnw', 'gradlew', 'pom.xml', 'build.gradle'),
    settings = {
      java = {
        configuration = {
          -- TODO: Use some sort of local configuration
          runtimes = {
            {
              name = 'JavaSE-17',
              path = os.getenv 'JAVA_17_HOME',
            },
            {
              name = 'JavaSE-21',
              path = os.getenv 'JAVA_21_HOME',
            },
          },
          annotationProcessing = {
            enabled = true,
            include = { 'org.mapstruct.*', 'lombok.*' },
          },
        },
      },
    },
  }
end

local function build_lua_ls_config()
  return {
    settings = {
      Lua = {
        completion = {
          callSnippet = 'Replace',
        },
      },
    },
  }
end

return {
  {
    'neovim/nvim-lspconfig',
    dependencies = {
      { 'williamboman/mason.nvim', opts = {} },
      'williamboman/mason-lspconfig.nvim',
      'WhoIsSethDaniel/mason-tool-installer.nvim',
      -- Useful status updates for LSP.
      {
        'j-hui/fidget.nvim',
        opts = {
          progress = {
            display = {
              done_ttl = 1,
            },
          },
          notification = {
            window = {
              winblend = 50,
            },
          },
        },
      },
      -- Allows extra capabilities provided by nvim-cmp
      'hrsh7th/cmp-nvim-lsp',
    },
    config = function()
      vim.api.nvim_create_autocmd('LspAttach', {
        group = vim.api.nvim_create_augroup('kickstart-lsp-attach', { clear = true }),
        callback = function(event)
          local map = function(keys, func, desc, mode)
            mode = mode or 'n'
            vim.keymap.set(mode, keys, func, { buffer = event.buf, desc = '' .. desc })
          end

          map('gd', require('telescope.builtin').lsp_definitions, '[g]oto [d]efinition')
          map('gr', require('telescope.builtin').lsp_references, '[g]oto [r]eferences')
          map('gI', require('telescope.builtin').lsp_implementations, '[g]oto [I]mplementation')
          map('gD', vim.lsp.buf.declaration, '[g]oto [D]eclaration')
          map('<leader>ca', vim.lsp.buf.code_action, '[c]ode [a]ction')
          map('<M-CR>', vim.lsp.buf.code_action, '[c]ode [a]ction')
        end,
      })

      if vim.g.have_nerd_font then
        local signs = { ERROR = '', WARN = '', INFO = '', HINT = '' }
        local diagnostic_signs = {}
        for type, icon in pairs(signs) do
          diagnostic_signs[vim.diagnostic.severity[type]] = icon
        end
        vim.diagnostic.config { signs = { text = diagnostic_signs } }
      end

      local capabilities = vim.lsp.protocol.make_client_capabilities()
      capabilities = vim.tbl_deep_extend('force', capabilities, require('cmp_nvim_lsp').default_capabilities())

      local servers = {
        rust_analyzer = {},
        lua_ls = build_lua_ls_config(),
        jdtls = build_jdtls_config(),
        ts_ls = {},
        astro = {},
        stylua = {},
        markdownlint = {},
      }
      local ensure_installed = vim.tbl_keys(servers or {})

      require('mason-tool-installer').setup { ensure_installed = ensure_installed }

      require('mason-lspconfig').setup {
        handlers = {
          function(server_name)
            local server = servers[server_name] or {}
            server.capabilities = vim.tbl_deep_extend('force', {}, capabilities, server.capabilities or {})
            require('lspconfig')[server_name].setup(server)
          end,
        },
      }

      require('lspconfig').qmlls.setup {
        cmd = { 'qmlls6' },
      }
    end,
  },
  {
    -- `lazydev` configures Lua LSP for your Neovim config, runtime and plugins
    -- used for completion, annotations and signatures of Neovim apis
    'folke/lazydev.nvim',
    ft = 'lua',
    opts = {
      library = {
        -- Load luvit types when the `vim.uv` word is found
        { path = '${3rd}/luv/library', words = { 'vim%.uv' } },
      },
    },
  },
  {
    'smjonas/inc-rename.nvim',
    opts = {},
    config = function(_, opts)
      require('inc_rename').setup(opts)
      vim.keymap.set('n', '<leader>cr', function()
        return ':IncRename ' .. vim.fn.expand '<cword>'
      end, { expr = true })
    end,
  },
  { 'mfussenegger/nvim-jdtls' },
  {
    'nvim-flutter/flutter-tools.nvim',
    lazy = false,
    dependencies = {
      'nvim-lua/plenary.nvim',
      'stevearc/dressing.nvim', -- optional for vim.ui.select
    },
    config = true,
  },
}
