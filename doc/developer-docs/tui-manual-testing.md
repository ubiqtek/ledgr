# Driving the TUI for manual testing

`ledgr`'s TUI (`app.rs`/`ui.rs`/`main.rs`) takes over the terminal
(`EnterAlternateScreen`, raw mode), so it can't be exercised by piping
input/output through a normal shell command the way the CLI subcommands
(`ledgr import`, `ledgr status`) can. `tmux` gives a scriptable terminal to
launch it in, send keystrokes to, and capture the rendered screen from —
useful whenever a change touches a screen, not just a `Db`/`derive`
function, and unit tests alone wouldn't show whether it actually renders
and navigates correctly.

## Pattern

```sh
# Launch in a detached session, sized to something realistic.
tmux new-session -d -s ledgr_test -x 200 -y 50 "cargo run 2>/tmp/ledgr_run.log"
sleep 3   # give cargo time to build + the app time to start

# See what's on screen.
tmux capture-pane -t ledgr_test -p

# Drive it — same keys a user would press.
tmux send-keys -t ledgr_test "m"
tmux send-keys -t ledgr_test "Enter"
tmux send-keys -t ledgr_test "Escape"
tmux capture-pane -t ledgr_test -p

# Tear down when done.
tmux send-keys -t ledgr_test "q"
tmux kill-session -t ledgr_test 2>/dev/null
```

Notes:
- `tmux send-keys` takes key names, not just literal characters — `Enter`,
  `Escape`, `Up`/`Down`, `C-d` (Ctrl-d) all work as you'd expect for
  `ledgr`'s keybindings (see `?` in-app or `draw_help` in `ui.rs`).
- This runs against the **real** local `ledgr.db`
  (`~/.local/share/ledgr/ledgr.db`) — the same database `cargo run`
  normally opens. It's a read-heavy TUI (browsing, no data-mutating
  screens yet), so this has been safe so far; if a future screen adds
  editing, back up the real DB first (as already practised for direct DB
  surgery — see the plan's session notes) or point at a scratch copy.
- Always `capture-pane` after a `send-keys` that's expected to change the
  screen, and actually look at the output — this is what makes it a real
  functional check rather than "the process didn't crash".

Used to verify the Monthly Gap screen and its per-month spend drill-down
end-to-end (2026-07-12) — see `doc/planning/plan.md`.
