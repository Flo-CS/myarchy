local theme_file_path = vim.fn.stdpath('config') .. '/.theme'
local uv = vim.uv

local function read_theme_file()
  local file = io.open(theme_file_path, 'r')
  if not file then
    return nil
  end
  local content = file:read('*line')
  file:close()
  return content
end

local function watch_theme_file_change(on_change)
  local handle
  local stopped = false

  local function cleanup()
    if handle then
      uv.fs_event_stop(handle)
      uv.close(handle)
      handle = nil
    end
  end

  local function start()
    if stopped then
      return
    end

    handle = uv.new_fs_event()
    if not handle then
      vim.schedule(function()
        vim.notify('Could not create fs_event handle', vim.log.levels.ERROR)
      end)
      return
    end

    uv.fs_event_start(handle, theme_file_path, {
      watch_entry = false,
      stat = false,
      recursive = false,
    }, function(err, filename, events)
      if err then
        vim.schedule(function()
          vim.notify('File watcher failed: ' .. err, vim.log.levels.ERROR)
        end)
        cleanup()
        return
      end

      vim.schedule(function()
        on_change(filename, events)
      end)

      cleanup()
      start()
    end)
  end

  start()

  return function()
    stopped = true
    cleanup()
  end
end

local function set_theme(theme)
  local ok = pcall(vim.cmd.colorscheme, theme)
  if not ok then
    vim.notify('Colorscheme ' .. theme .. ' not found!', vim.log.levels.WARN)
  end
end

-- Initial load
vim.defer_fn(function()
  local theme = read_theme_file()
  set_theme(theme)
end, 50)

-- Watch for changes
watch_theme_file_change(function()
  local theme = read_theme_file()
  set_theme(theme)
end)
