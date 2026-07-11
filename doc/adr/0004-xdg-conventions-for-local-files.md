# 4. Use XDG conventions for `ledgr`'s local files, not platform-native dirs

Date: 2026-07-11

## Status

Accepted

## Context

`ledgr` needs two persistent local files: a config file (currently just
`inbox_dir`, the directory it scans for downloaded statements) and its
SQLite database (`ledgr.db`). The `directories` crate gives
platform-native paths via `ProjectDirs` — on macOS that resolves to
`~/Library/Application Support/dev.ledgr.ledgr/{config.toml,ledgr.db}`,
the convention native GUI apps follow.

`ledgr` is a CLI tool, not a GUI app, and its primary user (its author)
manages personal tool configuration through a dotfiles repo, symlinked
into place — the standard pattern for that is targeting `~/.config/*`,
which most dotfiles managers and CLI tools already assume. Nesting
inside `~/Library/Application Support` works but is awkward to symlink
from a dotfiles repo and isn't where a terminal-first user would expect
to find or edit config or data by hand.

## Decision

Use the XDG Base Directory convention on every platform (not just
Linux), computed via `directories::BaseDirs::home_dir()` rather than
`ProjectDirs`:

- `Config::default_path()` → `~/.config/ledgr/config.toml`. Default
  `inbox_dir` (written on first run if no config exists) is
  `~/.config/ledgr/inbox`.
- `data_dir_db_path()` (`src/main.rs`) → `~/.local/share/ledgr/ledgr.db`.

## Consequences

- Config and data sit under a consistent, predictable pair —
  `~/.config/ledgr` and `~/.local/share/ledgr` — both symlinkable into a
  dotfiles repo the same way, without fighting a platform-native
  location.
- This is a deliberate deviation from macOS convention; if `ledgr` ever
  grows a GUI frontend, that frontend may want its own storage more
  aligned with platform norms rather than reusing these paths.
- No migration path was written for the brief window this shipped with
  `ProjectDirs`-based paths — the project has no released binary in use
  with a config file or database yet, so there's nothing to migrate
  from automatically. (In practice the one local database in existence
  during development was moved by hand.)
