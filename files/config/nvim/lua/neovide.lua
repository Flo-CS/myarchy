if vim.g.neovide then
  print 'Running in Neovide'
  vim.o.guifont = 'JetBrainsMono Nerd Font'
  vim.opt.linespace = 3
  -- Styling
  vim.g.neovide_scroll_animation_length = 0.1
  vim.g.neovide_cursor_animation_length = 0.075
  vim.g.neovide_cursor_trail_size = 0.1
  -- vim.g.neovide_transparency = 0.975
  -- vim.g.neovide_normal_opacity = 0.975
  -- Scale
  vim.g.neovide_scale_factor = 0.85
  local change_scale_factor = function(delta)
    vim.g.neovide_scale_factor = vim.g.neovide_scale_factor * delta
  end
  vim.keymap.set('n', '<C-=>', function()
    change_scale_factor(1.15)
  end)
  vim.keymap.set('n', '<C-->', function()
    change_scale_factor(1 / 1.15)
  end)
end
