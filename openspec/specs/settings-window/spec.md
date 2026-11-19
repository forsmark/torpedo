# settings-window Specification

## Purpose
The settings window lets the user edit the trigger and the aliases without editing the config file by hand. It never writes a bad config file, and the user can open it from the input source menu.

## Requirements

### Requirement: The settings window edits the trigger
The settings window SHALL show the trigger of the config file, or `::` when the file sets none, and SHALL let the user change it.

#### Scenario: The trigger field
- **GIVEN** a config file with the trigger `;;`
- **WHEN** the user opens the settings window
- **THEN** the trigger field shows `;;`

### Requirement: The settings window edits the aliases
The settings window SHALL list the aliases of the config file with their texts. It SHALL let the user add an alias, change an alias name or text, and delete an alias.

#### Scenario: The alias list
- **GIVEN** a config file with the aliases `lgtm = "👍"` and `shrug = '¯\_(ツ)_/¯'`
- **WHEN** the user opens the settings window
- **THEN** the window lists both aliases with their texts

#### Scenario: Add, change and delete an alias
- **GIVEN** the settings window is open
- **WHEN** the user adds the alias `ship` for 🚀, changes its text to 🛳, and deletes it again
- **THEN** each step is saved to the config file

### Requirement: The settings window shows a bad value at its field
When a value in the settings window breaks a rule of the config file, the window SHALL show the problem next to that field. It SHALL NOT write the config file while any value is bad.

#### Scenario: A bad trigger in the window
- **GIVEN** the settings window is open
- **WHEN** the user types `ab` in the trigger field
- **THEN** the window shows the problem next to the trigger field

#### Scenario: A bad value is not written
- **GIVEN** the settings window is open and the alias name field holds `looks good`
- **WHEN** the user applies the field
- **THEN** the window shows the problem next to the name field
- **AND** the config file is unchanged

### Requirement: Saving refuses a bad value
Saving SHALL check each value with the rules that `config-file` defines. When a value is bad, saving SHALL leave the config file unchanged and name the problem.

#### Scenario: Saving a bad trigger
- **GIVEN** a config file with the trigger `;;`
- **WHEN** `torpedo-settings --set-trigger ab` runs
- **THEN** it fails and names `trigger`
- **AND** the config file still holds the trigger `;;`

#### Scenario: Saving a bad alias
- **GIVEN** a config file with no aliases
- **WHEN** `torpedo-settings --set-alias "looks good=👍"` runs
- **THEN** it fails and names the alias `looks good`
- **AND** the config file is unchanged

### Requirement: Saving replaces the file in one step
Saving SHALL write the new config file next to the old one and rename it into place. A reader SHALL see either the old file or the new one, never part of one.

#### Scenario: The file is replaced by a rename
- **GIVEN** a config file
- **WHEN** `torpedo-settings --set-trigger ";;"` runs
- **THEN** the config file is a new file with the trigger `;;`
- **AND** no temp file is left in its folder

#### Scenario: A reader never sees part of a file
- **GIVEN** a reader that reads the config file again and again
- **WHEN** 20 saves with different aliases run
- **THEN** every read is a whole file that `torpedo-settings --check` accepts

### Requirement: Saving keeps what the window does not manage
Saving SHALL keep the comments, the keys and the tables of the config file that the settings window does not edit.

#### Scenario: A comment and an unknown key survive
- **GIVEN** a config file with a comment line, `skin_tone = "medium"` and the alias `lgtm = "👍"`
- **WHEN** `torpedo-settings --set-alias "ship=🚀"` runs
- **THEN** the config file still holds the comment line, `skin_tone = "medium"` and the alias `lgtm`
- **AND** it holds the alias `ship = "🚀"`

### Requirement: Saving creates the config file
When the config file or its folder does not exist, saving SHALL create them.

#### Scenario: The first save
- **GIVEN** `XDG_CONFIG_HOME` names an empty folder
- **WHEN** `torpedo-settings --set-alias "lgtm=👍"` runs
- **THEN** `$XDG_CONFIG_HOME/torpedo/config.toml` holds the alias `lgtm`

### Requirement: The settings command runs without a window
`torpedo-settings` SHALL open no window when it runs with `--check`, `--set-trigger`, `--set-alias` or `--remove-alias`. `--config <file>` SHALL name another config file. `--check` SHALL exit with status 0 for a good file and 1 for a bad one, and SHALL print each problem.

#### Scenario: Check a good file
- **GIVEN** a config file with the trigger `;;` and the alias `lgtm = "👍"`
- **WHEN** `torpedo-settings --check` runs with no display
- **THEN** it exits with status 0

#### Scenario: Check a bad file
- **GIVEN** a config file that does not parse
- **WHEN** `torpedo-settings --check` runs with no display
- **THEN** it exits with status 1 and prints the problem with its line

#### Scenario: Remove an alias
- **GIVEN** a config file with the aliases `lgtm` and `ship`
- **WHEN** `torpedo-settings --remove-alias ship` runs with no display
- **THEN** the config file holds the alias `lgtm` and not `ship`

#### Scenario: Another config file
- **GIVEN** a file `other.toml` with the trigger `;;`
- **WHEN** `torpedo-settings --config other.toml --set-trigger ":"` runs
- **THEN** `other.toml` holds the trigger `:`
- **AND** no file under `XDG_CONFIG_HOME` changed

### Requirement: The input source menu opens the settings window
The engine SHALL register an IBus property with the label "Torpedo settings" whenever an input context gains focus. When the user activates that property, the engine SHALL start the settings command. The settings command SHALL be the value of `TORPEDO_SETTINGS_COMMAND` when it is set, and otherwise `torpedo-settings` in the engine's own folder.

#### Scenario: The menu item
- **WHEN** an input context gains focus
- **THEN** the panel receives a property list with one item labelled "Torpedo settings"

#### Scenario: The menu item starts the settings command
- **GIVEN** `TORPEDO_SETTINGS_COMMAND` names a command that records each start
- **WHEN** the panel activates the property "Torpedo settings"
- **THEN** the command was started once

#### Scenario: The default settings command
- **GIVEN** `TORPEDO_SETTINGS_COMMAND` is not set, and the engine's folder holds a `torpedo-settings` that records each start
- **WHEN** the panel activates the property "Torpedo settings"
- **THEN** that `torpedo-settings` was started once

#### Scenario: Typing goes on while the settings command runs
- **GIVEN** `TORPEDO_SETTINGS_COMMAND` names a command that keeps running
- **WHEN** the panel activates the property "Torpedo settings", and the user types `::fire:` at a boundary
- **THEN** the app receives 🔥
