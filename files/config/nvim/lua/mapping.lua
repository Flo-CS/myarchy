vim.keymap.set('n', '<Esc>', '<cmd>nohlsearch<CR>', { desc = 'Clear search highlight' })
vim.keymap.set('n', '<Esc>', '<cmd>noh<CR>', { desc = 'Clear search highlight' })

vim.keymap.set('t', '<Esc><Esc>', '<C-\\><C-n>', { desc = 'Exit terminal mode' })

vim.keymap.set('n', '<left>', '<cmd>echo "Use h to move!!"<CR>')
vim.keymap.set('n', '<right>', '<cmd>echo "Use l to move!!"<CR>')
vim.keymap.set('n', '<up>', '<cmd>echo "Use k to move!!"<CR>')
vim.keymap.set('n', '<down>', '<cmd>echo "Use j to move!!"<CR>')

vim.keymap.set('n', '<C-h>', '<C-w><C-h>', { desc = 'Move focus to the left window' })
vim.keymap.set('n', '<C-l>', '<C-w><C-l>', { desc = 'Move focus to the right window' })
vim.keymap.set('n', '<C-j>', '<C-w><C-j>', { desc = 'Move focus to the lower window' })
vim.keymap.set('n', '<C-k>', '<C-w><C-k>', { desc = 'Move focus to the upper window' })

vim.keymap.set('i', '<C-j>', '<down>', { noremap = false, desc = 'Move down in insert mode' })
vim.keymap.set('i', '<C-k>', '<up>', { noremap = false, desc = 'Move up in insert mode' })
vim.keymap.set('i', '<C-h>', '<left>', { noremap = false, desc = 'Move left in insert mode' })
vim.keymap.set('i', '<C-l>', '<right>', { noremap = false, desc = 'Move right in insert mode' })

vim.keymap.set('n', ']<CR>', 'o<Esc>', { desc = 'Insert newline below' })
vim.keymap.set('n', '[<CR>', 'O<Esc>', { desc = 'Insert newline above' })

vim.keymap.set('n', '<C-a>', 'ggVG', { desc = 'Select whole buffer' })
