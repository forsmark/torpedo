# usage Specification

## Purpose
Usage lets torpedo learn which emoji and alias texts the user commits, so the search can rank them first. It lives in the usage file, apart from the config file, and a missing or broken usage file never stops typing.

## Requirements

### Requirement: The usage file location
The usage file SHALL be `$XDG_STATE_HOME/torpedo/usage.toml`. When `XDG_STATE_HOME` is unset, empty or not an absolute path, the usage file SHALL be `~/.local/state/torpedo/usage.toml`.

#### Scenario: The usage file under XDG_STATE_HOME
- **GIVEN** `XDG_STATE_HOME` names an empty folder
- **WHEN** the user types `::fire:` at a boundary
- **THEN** `$XDG_STATE_HOME/torpedo/usage.toml` holds an entry for 🔥

#### Scenario: The default usage file location
- **GIVEN** `XDG_STATE_HOME` is unset, `HOME` names a folder, and `$HOME/.local/state/torpedo/usage.toml` exists
- **WHEN** `torpedo-settings --clear-usage` runs
- **THEN** `$HOME/.local/state/torpedo/usage.toml` no longer exists

### Requirement: A commit records usage
While `remember_usage` is `true`, each time the engine commits an emoji or an alias text from a search, it SHALL add 1 to the usage score of that text and write the usage file. It SHALL write the usage file once for each such commit and at no other time.

#### Scenario: Enter records the emoji
- **GIVEN** no usage file exists
- **WHEN** the user types `::fire` at a boundary and presses Enter
- **THEN** the usage file holds 🔥 with the usage score 1

#### Scenario: The closing colon records an alias text
- **GIVEN** the config file has the alias `lgtm = "👍"` and no usage file exists
- **WHEN** the user types `::lgtm:` at a boundary
- **THEN** the usage file holds 👍 with the usage score 1

#### Scenario: Two commits of one emoji
- **GIVEN** no usage file exists
- **WHEN** the user types `::fire:`, a space and `::fire:`
- **THEN** the usage file holds 🔥 with a usage score of 2, give or take 0.01

#### Scenario: Typing a query writes nothing
- **GIVEN** no usage file exists
- **WHEN** the user types `::fir` at a boundary
- **THEN** no usage file exists

### Requirement: Plain text records nothing
When the engine commits plain text, it SHALL NOT change the usage file.

#### Scenario: Escape records nothing
- **GIVEN** no usage file exists
- **WHEN** the user types `::fi` at a boundary and presses Esc
- **THEN** the app receives `::fi`
- **AND** no usage file exists

#### Scenario: A query with no match records nothing
- **GIVEN** no usage file exists
- **WHEN** the user types `::fireq` at a boundary
- **THEN** no usage file exists

### Requirement: The usage score fades with time
A usage score SHALL halve for every 14 days since the last commit of its text. A commit SHALL add its 1 to the score as it has faded by then.

#### Scenario: A recent use beats an old one
- **GIVEN** a usage file with 👍 at the usage score 4, last committed 60 days ago, and 👎 at the usage score 1, last committed now
- **WHEN** the user types `::thumbs` at a boundary
- **THEN** the first candidate reads `👎 thumbsdown`

#### Scenario: A commit adds to the faded score
- **GIVEN** a usage file with 🔥 at the usage score 2, last committed 14 days ago
- **WHEN** the user types `::fire:` at a boundary
- **THEN** the usage file holds 🔥 with a usage score of 2, give or take 0.01, last committed now

### Requirement: The usage file keeps at most 200 entries
After the engine writes the usage file, it SHALL hold at most 200 entries. When a commit adds a 201st entry, the engine SHALL drop the entry with the lowest usage score.

#### Scenario: A full usage file drops its lowest entry
- **GIVEN** a usage file with 200 entries, all last committed now, where 😀 has the usage score 0.5 and every other entry has 2
- **WHEN** the user types `::fire:` at a boundary
- **THEN** the usage file holds 200 entries
- **AND** it holds 🔥 and not 😀

#### Scenario: A usage file below the limit keeps every entry
- **GIVEN** a usage file with 199 entries that do not include 🔥
- **WHEN** the user types `::fire:` at a boundary
- **THEN** the usage file holds 200 entries

### Requirement: The usage file is replaced in one step
The engine SHALL write the usage file to a temp file in the same folder and rename it into place. A reader SHALL see either the old file or the new one, never part of one.

#### Scenario: The usage file is replaced by a rename
- **GIVEN** a usage file
- **WHEN** the user types `::fire:` at a boundary
- **THEN** the usage file is a new file that holds 🔥
- **AND** no temp file is left in its folder

#### Scenario: A reader never sees part of a usage file
- **GIVEN** a reader that reads the usage file again and again
- **WHEN** the user commits 20 emoji
- **THEN** every read is a whole usage file

### Requirement: A missing or broken usage file never stops typing
When the usage file is missing, does not parse or holds a bad entry, the engine SHALL use no usage. The next commit SHALL replace a broken usage file with one that holds only that commit. Before that, the engine SHALL rename the broken usage file to `usage.toml.broken` in the same folder, replacing an older one.

#### Scenario: A broken usage file at start
- **GIVEN** the usage file holds `[[used]` with no closing bracket
- **WHEN** the user types `::thumbs` at a boundary, presses Esc, and types a space and `::fire:`
- **THEN** the first candidate of `::thumbs` reads `👍 thumbsup`
- **AND** the app receives `::thumbs 🔥`

#### Scenario: A usage file with a bad entry
- **GIVEN** a usage file with 👎 at the usage score 5 and an entry with no text
- **WHEN** the user types `::thumbs` at a boundary
- **THEN** the first candidate reads `👍 thumbsup`

#### Scenario: A commit replaces a broken usage file
- **GIVEN** the usage file holds `[[used]` with no closing bracket
- **WHEN** the user types `::fire:` at a boundary
- **THEN** the usage file holds one entry, for 🔥

#### Scenario: A broken usage file is kept aside
- **GIVEN** the usage file holds `[[used]` with no closing bracket, and `usage.toml.broken` holds older text
- **WHEN** the user types `::fire:` at a boundary
- **THEN** `usage.toml.broken` holds `[[used]` with no closing bracket
- **AND** the usage file holds one entry, for 🔥

### Requirement: Turning usage off
When the config file sets `remember_usage = false`, the engine SHALL NOT write the usage file, and the search SHALL ignore the usage. The engine SHALL NOT delete the usage file.

#### Scenario: Usage off writes nothing
- **GIVEN** the config file sets `remember_usage = false` and no usage file exists
- **WHEN** the user types `::fire:` at a boundary
- **THEN** the app receives 🔥
- **AND** no usage file exists

#### Scenario: Usage off ignores the usage file
- **GIVEN** the config file sets `remember_usage = false`, and a usage file holds 👎 at the usage score 5
- **WHEN** the user types `::thumbs` at a boundary
- **THEN** the first candidate reads `👍 thumbsup`

#### Scenario: Usage off keeps the usage file
- **GIVEN** the config file sets `remember_usage = false`, and a usage file holds 👎
- **WHEN** the user types `::fire:` at a boundary
- **THEN** the usage file is unchanged

### Requirement: The engine reads a changed usage file
The engine SHALL read the usage file again when its change time, size or file identity differs from what the engine last read or wrote. It SHALL check on every focus-in and each time a search opens. A deleted usage file SHALL mean no usage.

#### Scenario: A new usage file when a search opens
- **GIVEN** the engine runs with no usage file, and the input context keeps its focus
- **WHEN** a usage file with 👎 at the usage score 5 appears, and the user types `::thumbs` at a boundary
- **THEN** the first candidate reads `👎 thumbsdown`

#### Scenario: A deleted usage file after focus-in
- **GIVEN** the user typed `::thumbsdown:` twice
- **WHEN** the usage file is deleted, the input context loses and regains focus, and the user types `::thumbs` at a boundary
- **THEN** the first candidate reads `👍 thumbsup`

#### Scenario: A commit after the usage file is deleted
- **GIVEN** the user typed `::thumbsdown:` twice
- **WHEN** the usage file is deleted, the input context loses and regains focus, and the user types `::fire:` at a boundary
- **THEN** the usage file holds one entry, for 🔥
