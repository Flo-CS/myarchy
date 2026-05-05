-- Keybindings
vim.g.mapleader = ' '
vim.g.maplocalleader = ' '
vim.opt.timeoutlen = 300 -- Sequence timeout

-- Update time
vim.opt.updatetime = 500

-- File manager
vim.g.loaded_netrw = 1
vim.g.loaded_netrwPlugin = 1

-- Files and history
vim.opt.undofile = true
vim.opt.undolevels = 100000

-- Backup
vim.opt.backup = false
vim.opt.writebackup = false

-- Theme
vim.opt.termguicolors = true

-- Line numbers and gutter
vim.opt.number = true
vim.opt.relativenumber = true

-- Cursor
vim.opt.cursorline = true
vim.opt.cursorlineopt = 'screenline'
vim.opt.cursorcolumn = false

-- Scrolling
vim.opt.scrolloff = 8 -- Lines of context around cursor
vim.opt.sidescrolloff = 8 -- Columns of context around cursor

-- Whitespace
vim.opt.list = true -- Show whitespace characters
vim.opt.listchars = { tab = '» ', trail = '·', nbsp = '␣' }

-- Indentation
vim.opt.autoindent = true
vim.opt.copyindent = true
vim.opt.breakindent = true
vim.opt.tabstop = 4 -- Default width in columns of a tab
vim.opt.shiftwidth = 4 -- Default spaces per indent operation
vim.opt.shiftround = true -- Round indent operation to shiftwidth

-- Line wrapping
vim.opt.wrap = false
vim.opt.linebreak = true

-- Word boundaries
vim.opt.iskeyword:append('-')

-- Code folding
vim.opt.foldlevel = 99
vim.opt.foldnestmax = 4

-- Mouse
vim.opt.mouse = 'a' -- Enable mouse in all modes

-- Clipboard
vim.opt.clipboard = 'unnamedplus' -- Sync with system clipboard

-- Search and replace
vim.opt.ignorecase = true -- Case-insensitive search
vim.opt.smartcase = true -- ...unless query has capitals
vim.opt.inccommand = 'nosplit' -- Show live preview of substitutions

-- Spelling
vim.opt.spell = true
vim.opt.spelllang = 'en_us'
