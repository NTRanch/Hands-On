-- init.lua — dedicated alternate Neovim profile for hands-on
-- (IDEA-0093), the NSFW-lane learning game.
--
-- Install (local machine only — this directory is NOT part of
-- any repository, and this profile is deliberately separate from
-- the main config):
--
--   1. Copy this file to  ~/.config/hands-on/init.lua
--      (pick any <appname> you like; "hands-on" is the example)
--   2. Edit GAME_WORKDIR below to the game's workdir path.
--   3. Launch:  NVIM_APPNAME=hands-on nvim
--
-- Your main config never loads this file. The game's
-- commands exist only inside this profile's sessions.

-- EDIT ME: absolute path of your hands-on clone.
local GAME_WORKDIR = vim.fn.expand('~/hands-on')

-- A minimal standalone profile: sensible defaults only, no
-- plugins, nothing inherited from any other config.
vim.o.number = true
vim.o.termguicolors = true
vim.o.mouse = 'a'

-- Load the game's launcher from its own workdir.
local launcher = GAME_WORKDIR .. '/milking.nvim.lua'
if vim.fn.filereadable(launcher) == 1 then
    dofile(launcher)
    vim.keymap.set('n', '<leader>mg', '<cmd>MilkingGame<cr>', { desc = 'hands-on TUI' })
    vim.keymap.set('n', '<leader>mh', '<cmd>MilkingGameHtml<cr>', { desc = 'hands-on HTML' })
else
    vim.notify('hands-on launcher not found: ' .. launcher, vim.log.levels.ERROR)
end
