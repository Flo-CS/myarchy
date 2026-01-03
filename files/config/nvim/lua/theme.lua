local theme_file_path = vim.fn.stdpath 'config' .. '/.theme'
local uv = vim.uv

local function read_theme_file(path)
  local file = io.open(path, 'r')
  if not file then
    return 'default'
  end
  local content = file:read '*line'
  file:close()
  return content
end

local function set_theme(theme)
  local ok = pcall(vim.cmd.colorscheme, theme)
  if not ok then
    vim.notify('Colorscheme ' .. theme .. ' not found!', vim.log.levels.WARN)
  end
end

local function watch_theme_change()
  local handle = uv.new_fs_event()

  local unwatch_cb = function()
    if handle then
      uv.fs_event_stop(handle)
    end
  end

  local event_cb = function(err)
    if err then
      error 'Theme file watcher failed'
      unwatch_cb()
    else
      vim.schedule(function()
        local theme = read_theme_file(theme_file_path)
        set_theme(theme)
        unwatch_cb()
        watch_theme_change()
      end)
    end
  end

  if handle then
    uv.fs_event_start(handle, theme_file_path, {
      watch_entry = false,
      stat = false,
      recursive = false,
    }, event_cb)
  end

  return handle
end

-- TODO: Not the best way to handle the initial theme load
vim.defer_fn(function()
  local theme = read_theme_file(theme_file_path)
  set_theme(theme)
end, 50)

watch_theme_change()
