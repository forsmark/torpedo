# Spec Delta

## MODIFIED Requirements

### Requirement: A missing config file means defaults
When the config file does not exist, the engine SHALL use the trigger `::`, no aliases, and the default of each other key.

#### Scenario: No config file
- **GIVEN** no config file exists
- **WHEN** the user types `::fire:` at a boundary
- **THEN** the app receives 🔥

### Requirement: Unknown keys are ignored
The engine and the settings window SHALL ignore keys and tables of the config file that they do not know.

#### Scenario: A key from a later version
- **GIVEN** the config file sets `theme = "dark"`, a table `[later]` and the trigger `;;`
- **WHEN** the user types `;;fire:` at a boundary
- **THEN** the app receives 🔥

## ADDED Requirements

### Requirement: The skin_tone value
The `skin_tone` key SHALL be one of the strings `none`, `light`, `medium-light`, `medium`, `medium-dark` and `dark`. Any other value SHALL make the config file bad. Without the key, the skin tone SHALL be `none`.

#### Scenario: A known skin tone
- **GIVEN** a config file with `skin_tone = "medium-dark"`
- **WHEN** `torpedo-settings --check` checks it
- **THEN** the file is good

#### Scenario: A skin tone in upper case
- **GIVEN** a config file with `skin_tone = "Medium"`
- **WHEN** `torpedo-settings --check` checks it
- **THEN** the file is bad, and the problem names `skin_tone`

#### Scenario: A skin tone that is not a string
- **GIVEN** a config file with `skin_tone = 3`
- **WHEN** `torpedo-settings --check` checks it
- **THEN** the file is bad, and the problem names `skin_tone`

#### Scenario: No skin_tone key
- **GIVEN** a config file with only the trigger `::`
- **WHEN** the user types `::wave:` at a boundary
- **THEN** the app receives 👋 with no modifier

### Requirement: The remember_usage value
The `remember_usage` key SHALL be a boolean. Any other value SHALL make the config file bad. Without the key, `remember_usage` SHALL be `true`.

#### Scenario: Usage turned off
- **GIVEN** a config file with `remember_usage = false`
- **WHEN** `torpedo-settings --check` checks it
- **THEN** the file is good

#### Scenario: remember_usage as a string
- **GIVEN** a config file with `remember_usage = "no"`
- **WHEN** `torpedo-settings --check` checks it
- **THEN** the file is bad, and the problem names `remember_usage`

#### Scenario: No remember_usage key
- **GIVEN** a config file with only the trigger `::`, and no usage file exists
- **WHEN** the user types `::fire:` at a boundary
- **THEN** the usage file holds 🔥
