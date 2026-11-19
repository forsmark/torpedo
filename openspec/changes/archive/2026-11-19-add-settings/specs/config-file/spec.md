## Purpose

The config file holds the user's trigger and aliases. The engine reads it, and the settings window writes it. A bad or missing file never stops typing.

## ADDED Requirements

### Requirement: The config file location
The config file SHALL be `$XDG_CONFIG_HOME/torpedo/config.toml`. When `XDG_CONFIG_HOME` is unset, empty or not an absolute path, the config file SHALL be `~/.config/torpedo/config.toml`.

#### Scenario: The config file under XDG_CONFIG_HOME
- **GIVEN** `XDG_CONFIG_HOME` names a folder whose `torpedo/config.toml` sets the trigger `;;`
- **WHEN** the user types `;;fire:` at a boundary
- **THEN** the app receives 🔥

#### Scenario: The default location
- **GIVEN** `XDG_CONFIG_HOME` is unset and `HOME` names a folder
- **WHEN** `torpedo-settings --check` runs
- **THEN** it checks `$HOME/.config/torpedo/config.toml`

### Requirement: A missing config file means defaults
When the config file does not exist, the engine SHALL use the trigger `::` and no aliases.

#### Scenario: No config file
- **GIVEN** no config file exists
- **WHEN** the user types `::fire:` at a boundary
- **THEN** the app receives 🔥

### Requirement: A bad config file never stops typing
When the config file does not parse, or holds a bad value, the engine SHALL apply none of it. It SHALL keep the last good config it read, or the defaults when it read none.

#### Scenario: A broken file at start
- **GIVEN** the config file holds `trigger = ";;` with no closing quote
- **WHEN** the user types `::fire:` at a boundary
- **THEN** the app receives 🔥

#### Scenario: A bad value at start
- **GIVEN** the config file sets the trigger `;;` and an alias `lgtm` with an empty text
- **WHEN** the user types `::fire:` at a boundary
- **THEN** the app receives 🔥

#### Scenario: A broken file after a good one
- **GIVEN** the engine read a config file with the alias `lgtm = "👍"`
- **WHEN** the file changes to text that does not parse, and the user types `::lgtm:` at a boundary
- **THEN** the app receives 👍

### Requirement: The engine logs a bad config file
When the engine finds a bad config file and `TORPEDO_DEBUG` is set, it SHALL write the file's path and the problem to stderr.

#### Scenario: The log of a bad trigger
- **GIVEN** `TORPEDO_DEBUG=1` and a config file that sets the trigger `ab`
- **WHEN** the engine reads the config file
- **THEN** its stderr names the config file and the key `trigger`

### Requirement: Unknown keys are ignored
The engine and the settings window SHALL ignore keys and tables of the config file that they do not know.

#### Scenario: A key from a later version
- **GIVEN** the config file sets `skin_tone = "medium"`, a table `[recents]` and the trigger `;;`
- **WHEN** the user types `;;fire:` at a boundary
- **THEN** the app receives 🔥

### Requirement: The trigger value
The `trigger` key SHALL be a string of 1 to 3 characters. Each character SHALL be one that is not a query character, not whitespace and not a control character. Any other value SHALL make the config file bad.

#### Scenario: Two semicolons
- **GIVEN** a config file with `trigger = ";;"`
- **WHEN** `torpedo-settings --check` checks it
- **THEN** the file is good

#### Scenario: One colon
- **GIVEN** a config file with `trigger = ":"`
- **WHEN** `torpedo-settings --check` checks it
- **THEN** the file is good

#### Scenario: A trigger with a letter
- **GIVEN** a config file with `trigger = ":a"`
- **WHEN** `torpedo-settings --check` checks it
- **THEN** the file is bad, and the problem names `trigger`

#### Scenario: A trigger with a query sign
- **GIVEN** a config file with `trigger = "+"`
- **WHEN** `torpedo-settings --check` checks it
- **THEN** the file is bad, and the problem names `trigger`

#### Scenario: A trigger with a space
- **GIVEN** a config file with `trigger = ": "`
- **WHEN** `torpedo-settings --check` checks it
- **THEN** the file is bad, and the problem names `trigger`

#### Scenario: A trigger of four characters
- **GIVEN** a config file with `trigger = "::::"`
- **WHEN** `torpedo-settings --check` checks it
- **THEN** the file is bad, and the problem names `trigger`

#### Scenario: An empty trigger
- **GIVEN** a config file with `trigger = ""`
- **WHEN** `torpedo-settings --check` checks it
- **THEN** the file is bad, and the problem names `trigger`

#### Scenario: A trigger that is not a string
- **GIVEN** a config file with `trigger = 2`
- **WHEN** `torpedo-settings --check` checks it
- **THEN** the file is bad, and the problem names `trigger`

### Requirement: Alias names
Each key of the `[aliases]` table SHALL be an alias name of one or more query characters. Two alias names that differ only in the case of letters SHALL make the config file bad, because the search ignores case.

#### Scenario: A name with a plus sign
- **GIVEN** a config file with `"lgtm+" = "👍"` under `[aliases]`
- **WHEN** `torpedo-settings --check` checks it
- **THEN** the file is good

#### Scenario: A name with a space
- **GIVEN** a config file with `"looks good" = "👍"` under `[aliases]`
- **WHEN** `torpedo-settings --check` checks it
- **THEN** the file is bad, and the problem names the alias `looks good`

#### Scenario: Two names that differ in case
- **GIVEN** a config file with `lgtm = "👍"` and `LGTM = "✅"` under `[aliases]`
- **WHEN** `torpedo-settings --check` checks it
- **THEN** the file is bad, and the problem names the alias `LGTM`

### Requirement: Alias texts
Each value of the `[aliases]` table SHALL be a string of 1 to 16 Unicode code points with no control character. Any other value SHALL make the config file bad.

#### Scenario: A text emoji
- **GIVEN** a config file with `shrug = '¯\_(ツ)_/¯'` under `[aliases]`
- **WHEN** `torpedo-settings --check` checks it
- **THEN** the file is good

#### Scenario: An empty text
- **GIVEN** a config file with `lgtm = ""` under `[aliases]`
- **WHEN** `torpedo-settings --check` checks it
- **THEN** the file is bad, and the problem names the alias `lgtm`

#### Scenario: A text of 17 code points
- **GIVEN** a config file with `long = "abcdefghijklmnopq"` under `[aliases]`
- **WHEN** `torpedo-settings --check` checks it
- **THEN** the file is bad, and the problem names the alias `long`

#### Scenario: A text with a line break
- **GIVEN** a config file with `two = "a\nb"` under `[aliases]`
- **WHEN** `torpedo-settings --check` checks it
- **THEN** the file is bad, and the problem names the alias `two`

#### Scenario: A text that is not a string
- **GIVEN** a config file with `one = 1` under `[aliases]`
- **WHEN** `torpedo-settings --check` checks it
- **THEN** the file is bad, and the problem names the alias `one`

### Requirement: The engine reads a changed config file
The engine SHALL read the config file again when its change time, size or file identity differs from the last read. It SHALL check on every focus-in and each time a search opens. A deleted config file SHALL mean the defaults.

#### Scenario: A new alias when a search opens
- **GIVEN** the engine read a config file with no aliases, and the input context keeps its focus
- **WHEN** the file gains the alias `lgtm = "👍"`, and the user types `::lgtm:` at a boundary
- **THEN** the app receives 👍

#### Scenario: A new trigger after focus-in
- **GIVEN** the engine read a config file with the trigger `::`
- **WHEN** the file changes to the trigger `;;`, the input context loses and regains focus, and the user types `;;fire:` at a boundary
- **THEN** the app receives 🔥

#### Scenario: A deleted config file
- **GIVEN** the engine read a config file with the trigger `;;`
- **WHEN** the file is deleted, the input context loses and regains focus, and the user types `::fire:` at a boundary
- **THEN** the app receives 🔥

### Requirement: A new trigger applies from the next search
When the engine reads a new trigger while text is held, it SHALL finish the held trigger prefix or search with the trigger the user typed. The new trigger SHALL apply from the next key that finds no text held.

#### Scenario: The trigger changes as a search opens
- **GIVEN** the engine read a config file with the trigger `::`, and the input context keeps its focus
- **WHEN** the file changes to the trigger `;;`, and the user types `::` at a boundary and then Esc
- **THEN** the app receives `::`
- **AND** a search opened with the preedit `::`

#### Scenario: The new trigger after the search
- **GIVEN** the engine read a config file with the trigger `::`, and the input context keeps its focus
- **WHEN** the file changes to the trigger `;;`, and the user types `::`, Esc, a space and `;;fire:`
- **THEN** the app receives `:: 🔥`
