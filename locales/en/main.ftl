## Start & Help
start-welcome = Hey {$first_name}! I'm {$bot_name}, a powerful group management bot built with Rust.
help-header = Ferris Bot Help - Select a module below to see available commands.

## Admin
admin-promote-success = {$name} has been promoted!
admin-demote-success = {$name} has been demoted!
admin-no-permission = You don't have permission to use this command.
admin-bot-no-permission = I don't have the required permissions.

## Bans
ban-success = {$name} has been banned!
ban-reason = Reason: {$reason}
unban-success = User has been unbanned. They can now rejoin.
kick-success = User has been kicked!

## Mutes
mute-success = {$name} has been muted!
unmute-success = User has been unmuted!
tmute-success = User has been muted for {$time}!

## Warns
warn-success = {$name} has been warned! ({$count}/{$limit})
warn-exceeded = {$name} has been {$action}! Reached {$count}/{$limit} warnings.
warns-none = This user has no warnings.
warns-reset = Warnings have been reset for this user.
warn-limit-set = Warn limit set to {$limit}.
warn-mode-set = Warn mode set to {$mode}.

## Notes
note-saved = Note {$name} saved!
note-not-found = Note {$name} not found.
note-deleted = Note {$name} deleted!
notes-empty = No notes saved in this chat.

## Filters
filter-added = Filter {$keyword} added! I'll reply when someone says it.
filter-not-found = Filter {$keyword} not found.
filter-removed = Filter {$keyword} removed!
filters-empty = No filters set in this chat.
filters-all-removed = All filters have been removed!

## Welcome
welcome-enabled = Welcome messages enabled!
welcome-disabled = Welcome messages disabled!
welcome-updated = Welcome message updated!
goodbye-enabled = Goodbye messages enabled!
goodbye-disabled = Goodbye messages disabled!
goodbye-updated = Goodbye message updated!

## Rules
rules-empty = No rules set for this chat yet.
rules-updated = Rules updated successfully!
rules-cleared = Rules have been cleared!

## Blacklist
blacklist-empty = No blacklisted words in this chat.
blacklist-added = {$word} added to blacklist!
blacklist-removed = {$word} removed from blacklist!
blacklist-not-found = That word is not in the blacklist.
blacklist-mode-set = Default blacklist mode set to {$mode}.

## Purge
purge-complete = Purge complete! Deleted: {$count}
purge-no-reply = Reply to a message to start purging from.

## Pins
pin-success = Message pinned!
unpin-success = Message unpinned!
unpinall-success = All messages unpinned!

## Antiflood
flood-disabled = Antiflood has been disabled.
flood-set = Antiflood set to {$count} messages.
flood-mode-set = Antiflood mode set to {$mode}.
flood-action = {$name} has been {$action} for flooding!

## Disable
command-disabled = Command /{$command} has been disabled.
command-enabled = Command /{$command} has been enabled.
command-not-disabled = /{$command} was not disabled.
disabled-empty = No commands are disabled in this chat.

## Language
lang-set = Language set to {$language}.

## Errors
error-no-user = Please specify a user (reply or provide ID).
error-cant-action-admin = I can't perform this action on an admin/sudo user!
error-need-admin = You need to be an admin to use this command.
error-bot-no-restrict = I don't have permission to restrict users.
error-user-no-restrict = You don't have permission to restrict users.
