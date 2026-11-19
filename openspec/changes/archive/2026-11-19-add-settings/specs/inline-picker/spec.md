## RENAMED Requirements

- FROM: `### Requirement: The trigger is two colons`
- TO: `### Requirement: The trigger is configurable`

## MODIFIED Requirements

### Requirement: The trigger is configurable
The trigger SHALL be the `trigger` value of the config file, which `config-file` defines in *The trigger value*. Without that value the trigger SHALL be `::`. The other scenarios of this spec use the default trigger `::`.

#### Scenario: Two colons open a search
- **GIVEN** the cursor is at a boundary and the config file sets no trigger
- **WHEN** the user types `::`
- **THEN** a search opens with an empty query
- **AND** the preedit shows `::`

#### Scenario: Two semicolons open a search
- **GIVEN** the cursor is at a boundary and the config file sets the trigger `;;`
- **WHEN** the user types `;;`
- **THEN** a search opens with an empty query
- **AND** the preedit shows `;;`

#### Scenario: Two colons with another trigger
- **GIVEN** the cursor is at a boundary and the config file sets the trigger `;;`
- **WHEN** the user types `::fire`
- **THEN** the app receives `::fire` as typed
- **AND** no search opens

#### Scenario: A trigger of one colon
- **GIVEN** the cursor is at a boundary and the config file sets the trigger `:`
- **WHEN** the user types `:fire:`
- **THEN** the app receives 🔥

### Requirement: A colon after an exact shortcode commits it
When the query is an exact shortcode or an exact alias, `:` SHALL commit its emoji and close the search. This closing character SHALL be `:` whatever the trigger is.

#### Scenario: Slack style
- **WHEN** the user types `::fire:` at a boundary
- **THEN** the app receives 🔥

#### Scenario: A colon after a partial shortcode
- **WHEN** the user types `::fir:` at a boundary
- **THEN** the app receives `::fir:`

#### Scenario: A colon after an alias
- **GIVEN** the config file has the alias `lgtm = "👍"`
- **WHEN** the user types `::lgtm:` at a boundary
- **THEN** the app receives 👍

#### Scenario: A colon closes a search of another trigger
- **GIVEN** the config file sets the trigger `;;`
- **WHEN** the user types `;;fire:` at a boundary
- **THEN** the app receives 🔥

### Requirement: Another character closes the search
A character that is not a query character, other than the `:` after an exact shortcode or alias, SHALL close the search, commit the plain text, and then be handled as usual.

#### Scenario: A space closes the search
- **GIVEN** a search is open with the query `fi`
- **WHEN** the user types a space
- **THEN** the app receives `::fi `
- **AND** the search is closed

#### Scenario: A Danish letter closes the search
- **GIVEN** a search is open with the query `fi`
- **WHEN** the user types `æ`
- **THEN** the app receives `::fiæ`

### Requirement: The candidate list
The candidate list SHALL appear at the cursor when the query has at least one character and matches. It SHALL show 9 candidates per page with no labels. Each candidate SHALL show the emoji and its best matching shortcode or alias.

#### Scenario: No candidate list for an empty query
- **WHEN** the user types `::` at a boundary
- **THEN** no candidate list is shown

#### Scenario: The candidate list for a query
- **WHEN** the user types `::fi` at a boundary
- **THEN** the candidate list is shown with a page size of 9 and the first candidate selected

#### Scenario: Candidates have no labels
- **WHEN** the user types `::fi` at a boundary
- **THEN** the candidate list has no labels

#### Scenario: A candidate shows its emoji and shortcode
- **WHEN** the user types `::fire` at a boundary
- **THEN** the first candidate reads `🔥 fire`

#### Scenario: A candidate shows its alias
- **GIVEN** the config file has the alias `lgtm = "👍"`
- **WHEN** the user types `::lgtm` at a boundary
- **THEN** the first candidate reads `👍 lgtm`

### Requirement: Each emoji appears once
An emoji with several matching shortcodes or aliases SHALL appear once in the candidate list, under its best ranked shortcode or alias.

#### Scenario: An emoji with two matching shortcodes
- **WHEN** the user types `::fist` at a boundary
- **THEN** ✊ appears once, as `✊ fist`

#### Scenario: An emoji with a matching shortcode and alias
- **GIVEN** the config file has the alias `fireball = "🔥"`
- **WHEN** the user types `::fire` at a boundary
- **THEN** 🔥 appears once, as `🔥 fire`

### Requirement: Ranking
The candidate list SHALL rank an exact shortcode or alias first, then a shortcode or alias that starts with the query, then one with a `_`-separated word that starts with the query, then one that holds the query. Among equal ranks, aliases SHALL come first in the order of their names, and shortcodes SHALL keep gemoji's order.

#### Scenario: Exact before prefix before word prefix
- **WHEN** the user types `::fire` at a boundary
- **THEN** the candidates are 🔥 fire, then the shortcodes that start with `fire` such as fire_engine, then the shortcodes with a word that starts with `fire` such as heart_on_fire

#### Scenario: Prefix before substring
- **WHEN** the user types `::ire` at a boundary
- **THEN** 🇮🇪 ireland comes before 😫 tired_face

#### Scenario: Ties keep gemoji's order
- **WHEN** the user types `::thumbs` at a boundary
- **THEN** 👍 thumbsup comes before 👎 thumbsdown

#### Scenario: An exact alias before prefix matches
- **GIVEN** the config file has the alias `fi = "🇫🇮"`
- **WHEN** the user types `::fi` at a boundary
- **THEN** the first candidate reads `🇫🇮 fi`

#### Scenario: Aliases first among equal ranks
- **GIVEN** the config file has the aliases `thumbsboth = "👍👎"` and `thumbsall = "🙌"`
- **WHEN** the user types `::thumbs` at a boundary
- **THEN** the candidates are `🙌 thumbsall`, `👍👎 thumbsboth`, `👍 thumbsup` and `👎 thumbsdown`, in that order

### Requirement: A query with no match ends the search
When the query stops matching any shortcode or alias, the engine SHALL commit the plain text and close the search.

#### Scenario: No shortcode matches
- **GIVEN** a search is open with the query `fire`
- **WHEN** the user types `q`
- **THEN** the app receives `::fireq`
- **AND** the search is closed

#### Scenario: Only an alias matches
- **GIVEN** the config file has the alias `zzbot = "🤖"`
- **WHEN** the user types `::zzb` at a boundary
- **THEN** the search stays open and the first candidate reads `🤖 zzbot`

## ADDED Requirements

### Requirement: The trigger rules hold for any trigger
With any trigger, the trigger SHALL count only at a boundary, a proper prefix of the trigger SHALL be held, and Backspace SHALL remove a held trigger prefix.

#### Scenario: Another trigger after a word
- **GIVEN** the config file sets the trigger `;;` and the surrounding text before the cursor is `std`
- **WHEN** the user types `;;`
- **THEN** the app receives `;;` as typed
- **AND** no search opens

#### Scenario: The first character of another trigger is held
- **GIVEN** the config file sets the trigger `;;` and the cursor is at a boundary
- **WHEN** the user types `;`
- **THEN** the preedit shows `;`
- **AND** the engine commits nothing

#### Scenario: Backspace after the first character of another trigger
- **GIVEN** the config file sets the trigger `;;` and the user typed `;` at a boundary
- **WHEN** the user presses Backspace
- **THEN** the preedit is empty
- **AND** the app receives neither `;` nor Backspace

### Requirement: Aliases are searched like shortcodes
The search SHALL match the name of each alias in the config file like a shortcode of the alias's text. It SHALL ignore the case of the name. Committing an alias SHALL commit its whole text.

#### Scenario: An alias by its prefix
- **GIVEN** the config file has the alias `lgtm = "👍"`
- **WHEN** the user types `::lg` at a boundary
- **THEN** the candidate list shows `👍 lgtm`

#### Scenario: An alias in upper case
- **GIVEN** the config file has the alias `lgtm = "👍"`
- **WHEN** the user types `::LGTM` at a boundary
- **THEN** the first candidate reads `👍 lgtm`

#### Scenario: An alias for a text emoji
- **GIVEN** the config file has the alias `shrug = '¯\_(ツ)_/¯'`
- **WHEN** the user types `::shrug` at a boundary and presses Enter
- **THEN** the app receives `¯\_(ツ)_/¯`

### Requirement: An alias wins over a shortcode of the same name
When an alias has the name of a gemoji shortcode, the search SHALL match that name only to the alias.

#### Scenario: An alias named fire
- **GIVEN** the config file has the alias `fire = "🚒"`
- **WHEN** the user types `::fire:` at a boundary
- **THEN** the app receives 🚒

#### Scenario: The shortcode it hides
- **GIVEN** the config file has the alias `fire = "🚒"`
- **WHEN** the user types `::fire` at a boundary
- **THEN** the first candidate reads `🚒 fire`
- **AND** no candidate reads `🔥 fire`
