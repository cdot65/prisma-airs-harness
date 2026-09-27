// Aggregates all former standalone integration tests as modules.
#[cfg(unix)]
mod focus_palette;
#[cfg(unix)]
mod reconnect;
mod resize_reflow;
mod status_indicator;
mod vt100_history;
mod vt100_live_commit;
#[cfg(unix)]
mod worktree_stack;

#[cfg(unix)]
mod terminal_ssh;
#[cfg(unix)]
mod unicode_lists;

#[cfg(unix)]
mod unicode_math;

#[cfg(unix)]
mod transcript_find;
