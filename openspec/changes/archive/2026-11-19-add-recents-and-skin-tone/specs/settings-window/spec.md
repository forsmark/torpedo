# Spec Delta

## MODIFIED Requirements

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

#### Scenario: Saving a bad skin tone
- **GIVEN** a config file with `skin_tone = "medium"`
- **WHEN** `torpedo-settings --set-skin-tone purple` runs
- **THEN** it fails and names `skin_tone`
- **AND** the config file still holds `skin_tone = "medium"`

### Requirement: Saving keeps what the window does not manage
Saving SHALL keep the comments, the keys and the tables of the config file that the settings window does not edit.

#### Scenario: A comment and an unknown key survive
- **GIVEN** a config file with a comment line, `theme = "dark"` and the alias `lgtm = "👍"`
- **WHEN** `torpedo-settings --set-alias "ship=🚀"` runs
- **THEN** the config file still holds the comment line, `theme = "dark"` and the alias `lgtm`
- **AND** it holds the alias `ship = "🚀"`

### Requirement: The settings command runs without a window
`torpedo-settings` SHALL open no window when it runs with `--check`, `--set-trigger`, `--set-alias`, `--remove-alias`, `--set-skin-tone`, `--set-remember-usage` or `--clear-usage`. `--config <file>` SHALL name another config file. `--check` SHALL exit with status 0 for a good file and 1 for a bad one, and SHALL print each problem. `--clear-usage` SHALL delete the usage file, and SHALL succeed when there is none.

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

#### Scenario: Set the skin tone
- **GIVEN** a config file with the trigger `;;`
- **WHEN** `torpedo-settings --set-skin-tone medium-light` runs with no display
- **THEN** the config file holds `skin_tone = "medium-light"` and the trigger `;;`

#### Scenario: Turn usage off
- **GIVEN** a config file with no `remember_usage` key
- **WHEN** `torpedo-settings --set-remember-usage false` runs with no display
- **THEN** the config file holds `remember_usage = false`

#### Scenario: Clear the usage
- **GIVEN** `XDG_STATE_HOME` names a folder whose `torpedo/usage.toml` exists
- **WHEN** `torpedo-settings --clear-usage` runs with no display
- **THEN** it exits with status 0
- **AND** `$XDG_STATE_HOME/torpedo/usage.toml` no longer exists

#### Scenario: Clear the usage when there is none
- **GIVEN** `XDG_STATE_HOME` names an empty folder
- **WHEN** `torpedo-settings --clear-usage` runs with no display
- **THEN** it exits with status 0

## ADDED Requirements

### Requirement: The settings window edits the skin tone
The settings window SHALL offer each skin tone that `config-file` defines in *The skin_tone value*, each with a 👋 in that tone, and SHALL save the one the user picks.

#### Scenario: The skin tone selector
- **GIVEN** a config file with `skin_tone = "medium"`
- **WHEN** the user opens the settings window and picks the dark tone
- **THEN** before the pick, the selector showed the medium tone with 👋🏽
- **AND** the config file holds `skin_tone = "dark"`

### Requirement: The settings window edits remember_usage
The settings window SHALL show a switch for `remember_usage` and SHALL save its state when the user flips it.

#### Scenario: The usage switch
- **GIVEN** a config file with no `remember_usage` key
- **WHEN** the user opens the settings window and turns the switch off
- **THEN** before the flip, the switch was on
- **AND** the config file holds `remember_usage = false`

### Requirement: The settings window clears the usage
The "Clear usage" button SHALL ask for a confirmation in a dialog inside the settings window. The settings window SHALL delete the usage file only when the user confirms.

#### Scenario: Clear usage confirmed
- **GIVEN** a usage file exists
- **WHEN** the user presses "Clear usage" and confirms in the dialog
- **THEN** the usage file no longer exists

#### Scenario: Clear usage cancelled
- **GIVEN** a usage file exists
- **WHEN** the user presses "Clear usage" and cancels the dialog
- **THEN** the usage file is unchanged
