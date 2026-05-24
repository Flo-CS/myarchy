-- Better up/down
vim.keymap.set({ 'n', 'x' }, 'j', "v:count == 0 ? 'gj' : 'j'", { desc = 'Down', expr = true, silent = true })
vim.keymap.set({ 'n', 'x' }, '<Down>', "v:count == 0 ? 'gj' : 'j'", { desc = 'Down', expr = true, silent = true })
vim.keymap.set({ 'n', 'x' }, 'k', "v:count == 0 ? 'gk' : 'k'", { desc = 'Up', expr = true, silent = true })
vim.keymap.set({ 'n', 'x' }, '<Up>', "v:count == 0 ? 'gk' : 'k'", { desc = 'Up', expr = true, silent = true })

-- Move to window using the <ctrl> hjkl keys
vim.keymap.set('n', '<C-h>', '<C-w>h', { desc = 'Go to Left Window', remap = true })
vim.keymap.set('n', '<C-j>', '<C-w>j', { desc = 'Go to Lower Window', remap = true })
vim.keymap.set('n', '<C-k>', '<C-w>k', { desc = 'Go to Upper Window', remap = true })
vim.keymap.set('n', '<C-l>', '<C-w>l', { desc = 'Go to Right Window', remap = true })

-- Resize window using <ctrl> arrow keys
vim.keymap.set('n', '<C-Up>', '<cmd>resize +2<cr>', { desc = 'Increase Window Height' })
vim.keymap.set('n', '<C-Down>', '<cmd>resize -2<cr>', { desc = 'Decrease Window Height' })
vim.keymap.set('n', '<C-Left>', '<cmd>vertical resize -2<cr>', { desc = 'Decrease Window Width' })
vim.keymap.set('n', '<C-Right>', '<cmd>vertical resize +2<cr>', { desc = 'Increase Window Width' })

-- Move lines using <ctrl shift> hjkl keys
vim.keymap.set('n', '<C-S-j>', "<cmd>execute 'move .+' . v:count1<cr>==", { desc = 'Move Down' })
vim.keymap.set('n', '<C-S-k>', "<cmd>execute 'move .-' . (v:count1 + 1)<cr>==", { desc = 'Move Up' })
vim.keymap.set('i', '<C-S-j>', '<esc><cmd>m .+1<cr>==gi', { desc = 'Move Down' })
vim.keymap.set('i', '<C-S-k>', '<esc><cmd>m .-2<cr>==gi', { desc = 'Move Up' })
vim.keymap.set('v', '<C-S-j>', ":<C-u>execute \"'<,'>move '>+\" . v:count1<cr>gv=gv", { desc = 'Move Down' })
vim.keymap.set('v', '<C-S-k>', ":<C-u>execute \"'<,'>move '<-\" . (v:count1 + 1)<cr>gv=gv", { desc = 'Move Up' })

-- Better search navigation (https://github.com/mhinz/vim-galore#saner-behavior-of-n-and-n)
vim.keymap.set('n', 'n', "'Nn'[v:searchforward].'zv'", { expr = true, desc = 'Next Search Result' })
vim.keymap.set('x', 'n', "'Nn'[v:searchforward]", { expr = true, desc = 'Next Search Result' })
vim.keymap.set('o', 'n', "'Nn'[v:searchforward]", { expr = true, desc = 'Next Search Result' })
vim.keymap.set('n', 'N', "'nN'[v:searchforward].'zv'", { expr = true, desc = 'Prev Search Result' })
vim.keymap.set('x', 'N', "'nN'[v:searchforward]", { expr = true, desc = 'Prev Search Result' })
vim.keymap.set('o', 'N', "'nN'[v:searchforward]", { expr = true, desc = 'Prev Search Result' })

-- Clear search on escape
vim.keymap.set({ 'i', 'n', 's' }, '<esc>', function()
  vim.cmd('noh')
  return '<esc>'
end, { expr = true, desc = 'Escape and Clear hlsearch' })

-- Better indenting (don't lose selection when indenting)
vim.keymap.set('x', '<', '<gv')
vim.keymap.set('x', '>', '>gv')

-- Buffers
vim.keymap.set('n', '<C-a>', 'ggVG', { desc = 'Select whole buffer' })

-- Windows
vim.keymap.set('n', '<leader>w-', '<C-W>s', { desc = 'Split Window Below', remap = true })
vim.keymap.set('n', '<leader>w|', '<C-W>v', { desc = 'Split Window Right', remap = true })
vim.keymap.set('n', '<leader>wd', '<C-W>c', { desc = 'Delete Window', remap = true })

-- Files
vim.keymap.set('n', '<leader>fn', '<cmd>enew<cr>', { desc = 'New File' })

-- Session
vim.keymap.set('n', '<leader>qq', '<cmd>qa<cr>', { desc = 'Quit All' })
