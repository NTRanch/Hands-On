-- milking.nvim.lua — hands-on (IDEA-0093) launcher for Neovim.
--
-- NSFW-LANE FILE. This launcher belongs to the adult lane's
-- learning game. It is NOT part of any SFW configuration
-- and must never be added to one: do not copy it into your main
-- editor config, do not require it from any SFW-lane repo. Load it only
-- the two ways CURRICULUM.md documents:
--
--   (a) on demand, per session:
--         :luafile <workdir>/milking.nvim.lua
--       The commands below exist for that session only and
--       vanish when Neovim exits.
--   (b) a dedicated alternate profile (nvim-appname/init.lua in
--       this workdir), launched as:
--         NVIM_APPNAME=hands-on nvim
--       A local-only profile directory; your main config never
--       loads this file.
--
-- Commands defined:
--   :MilkingGame      open the hands-on TUI in a floating
--                     terminal window (the TUI twin on the same
--                     milking-core; the HTML/canvas build cannot
--                     render inside a text buffer)
--   :MilkingGameHtml  open game.html beside Neovim via vim.ui.open
--
-- Suggested keymaps (set them yourself, in the session/profile
-- that loaded this file — this file sets no global keymaps):
--   vim.keymap.set('n', '<leader>mg', '<cmd>MilkingGame<cr>')
--   vim.keymap.set('n', '<leader>mh', '<cmd>MilkingGameHtml<cr>')

local M = {}

-- Directory containing this file (the game workdir).
local function workdir()
    local src = debug.getinfo(1, 'S').source:sub(2) -- strip '@'
    return vim.fn.fnamemodify(src, ':p:h')
end

--- Binary candidates, first existing wins: a release build,
--- then a debug build, then PATH.
local function hands_on_bin()
    local dir = workdir()
    local candidates = {
        dir .. '/target/release/hands-on',
        dir .. '/target/debug/hands-on',
    }
    for _, path in ipairs(candidates) do
        if vim.fn.executable(path) == 1 then
            return path
        end
    end
    if vim.fn.executable('hands-on') == 1 then
        return 'hands-on'
    end
    return nil
end

--- Open the hands-on TUI in a centered floating terminal.
function M.open_game()
    local bin = hands_on_bin()
    if not bin then
        vim.notify(
            'hands-on binary not found — build it first: cargo build --release (in '
                .. workdir()
                .. ')',
            vim.log.levels.ERROR
        )
        return
    end
    local width = math.floor(vim.o.columns * 0.8)
    local height = math.floor(vim.o.lines * 0.8)
    local buf = vim.api.nvim_create_buf(false, true)
    local win = vim.api.nvim_open_win(buf, true, {
        relative = 'editor',
        width = width,
        height = height,
        row = math.floor((vim.o.lines - height) / 2),
        col = math.floor((vim.o.columns - width) / 2),
        style = 'minimal',
        border = 'rounded',
        title = ' hands-on — Level 1: Opal ',
        title_pos = 'center',
    })
    vim.fn.termopen(bin, {
        cwd = workdir(),
        on_exit = function()
            if vim.api.nvim_win_is_valid(win) then
                vim.api.nvim_win_close(win, true)
            end
        end,
    })
    vim.cmd.startinsert()
end

--- Open the HTML prototype beside Neovim (it cannot render
--- inside a text buffer; see CURRICULUM.md for the honest
--- boundary).
function M.open_html()
    local html = workdir() .. '/game.html'
    if vim.fn.filereadable(html) == 0 then
        vim.notify('game.html not found at ' .. html, vim.log.levels.ERROR)
        return
    end
    vim.ui.open(html)
end

vim.api.nvim_create_user_command('MilkingGame', M.open_game, {
    desc = 'hands-on: play the learning TUI in a floating terminal',
})
vim.api.nvim_create_user_command('MilkingGameHtml', M.open_html, {
    desc = 'hands-on: open the Level 1 HTML prototype beside Neovim',
})

return M
