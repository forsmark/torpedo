# inline-picker Specification

## Purpose

The inline picker lets the user type a trigger and a shortcode in any text field and insert the matching emoji, while all other typing reaches the app unchanged.

## Requirements

### Requirement: Keys the engine does not use pass through
The engine SHALL let every key it does not use reach the app unchanged. A key release SHALL always reach the app.

#### Scenario: A letter with nothing held passes through
- **GIVEN** no text is held
- **WHEN** the user types `a`
- **THEN** the key reaches the app
- **AND** the engine commits nothing and shows no preedit

#### Scenario: A key release passes through
- **GIVEN** a search is open with the query `fi`
- **WHEN** the user releases the `i` key
- **THEN** the release reaches the app
- **AND** the preedit still shows `::fi`

### Requirement: Ctrl, Alt and Super make a key pass through
A key pressed while Ctrl, Alt or Super is held SHALL pass through. This rule comes before the rules for Enter, Tab, Backspace, Esc, the arrows and the page keys. Shift and AltGr SHALL NOT make a key pass through, because the Danish layout needs them for `:` and for `@ £ $ { [ ] }`. Caps Lock and Num Lock SHALL NOT make a key pass through.

#### Scenario: Ctrl with a letter passes through
- **GIVEN** no text is held
- **WHEN** the user presses Ctrl+`c`
- **THEN** the key reaches the app

#### Scenario: Alt with a letter passes through
- **GIVEN** no text is held
- **WHEN** the user presses Alt+`f`
- **THEN** the key reaches the app

#### Scenario: Super with a letter passes through
- **GIVEN** no text is held
- **WHEN** the user presses Super+`a`
- **THEN** the key reaches the app

#### Scenario: Shift gives the colon of the trigger
- **GIVEN** the cursor is at a boundary
- **WHEN** the user types `:` twice as Shift+period
- **THEN** a search opens

#### Scenario: AltGr gives a character
- **GIVEN** a search is open with the query `fi`
- **WHEN** the user types `{` as AltGr+7
- **THEN** the app receives `::fi{`
- **AND** the search is closed

#### Scenario: Num Lock does not stop the trigger
- **GIVEN** the cursor is at a boundary and Num Lock is on
- **WHEN** the user types `::`
- **THEN** a search opens

### Requirement: A modifier key alone changes nothing
A press or release of a modifier key alone SHALL NOT change the held text.

#### Scenario: Shift pressed again between the colons
- **GIVEN** the cursor is at a boundary
- **WHEN** the user types `:`, presses and releases Shift, and types `:` again
- **THEN** a search opens

### Requirement: A dead key is held
A dead key SHALL be held, and the preedit SHALL show the spacing form of its accent. A dead key with no accent that torpedo knows SHALL pass through.

#### Scenario: The dead acute key
- **GIVEN** no text is held
- **WHEN** the user presses the dead acute key
- **THEN** the preedit shows `´`
- **AND** the key does not reach the app

#### Scenario: The accent is underlined
- **GIVEN** no text is held
- **WHEN** the user presses the dead diaeresis key
- **THEN** the preedit shows `¨` underlined

#### Scenario: An unknown dead key
- **GIVEN** no text is held
- **WHEN** the user presses the dead belowdot key
- **THEN** the key reaches the app

### Requirement: A dead key composes with the next character
When a dead key is held and the next character has a precomposed form with that accent, the engine SHALL commit the precomposed form.

#### Scenario: Acute and e
- **WHEN** the user presses the dead acute key and then `e`
- **THEN** the app receives `é`

#### Scenario: Diaeresis and u
- **WHEN** the user presses the dead diaeresis key and then `u`
- **THEN** the app receives `ü`

#### Scenario: Tilde and n
- **WHEN** the user presses the dead tilde key and then `n`
- **THEN** the app receives `ñ`

#### Scenario: Acute and upper case E
- **WHEN** the user presses the dead acute key and then Shift+`e`
- **THEN** the app receives `É`

### Requirement: A dead key alone gives its accent
When a dead key is held, a space or the same dead key again SHALL commit the spacing form of the accent.

#### Scenario: Dead key and space
- **WHEN** the user presses the dead acute key and then space
- **THEN** the app receives `´`

#### Scenario: The same dead key twice
- **WHEN** the user presses the dead circumflex key twice
- **THEN** the app receives `^`

### Requirement: A dead key that does not compose
When a dead key is held and the next character has no precomposed form with that accent, the engine SHALL commit the spacing accent and then handle the character as usual.

#### Scenario: Acute and q
- **WHEN** the user presses the dead acute key and then `q`
- **THEN** the app receives `´q`

### Requirement: A dead key before another key
When a dead key is held, a key without a character, other than Backspace and a modifier key alone, SHALL commit the spacing accent and then be handled as usual. Another dead key SHALL commit the first accent and be held itself. Backspace SHALL remove the held dead key.

#### Scenario: Dead key and Left
- **WHEN** the user presses the dead acute key and then Left
- **THEN** the app receives `´` and then Left

#### Scenario: Two different dead keys
- **WHEN** the user presses the dead acute key and then the dead diaeresis key
- **THEN** the app receives `´`
- **AND** the preedit shows `¨`

#### Scenario: Backspace after a dead key
- **WHEN** the user presses the dead acute key and then Backspace
- **THEN** the preedit is empty
- **AND** the app receives nothing

#### Scenario: Shift after a dead key
- **WHEN** the user presses the dead acute key, presses Shift, and types `e`
- **THEN** the app receives `É`

### Requirement: A dead key ends a trigger prefix or a search
A dead key while a trigger prefix or a search is held SHALL commit the held text as plain text, and the dead key SHALL then be held.

#### Scenario: A dead key while the first colon is held
- **GIVEN** the user typed `:` at a boundary
- **WHEN** the user presses the dead acute key
- **THEN** the app receives `:`
- **AND** the preedit shows `´`

#### Scenario: A dead key during a search
- **GIVEN** a search is open with the query `fi`
- **WHEN** the user presses the dead acute key
- **THEN** the app receives `::fi`
- **AND** the search is closed and the preedit shows `´`

### Requirement: Held text reaches the app before a key that passes through
When a key passes through while text is held, the engine SHALL commit the held text first. The app SHALL receive the held text before the key.

#### Scenario: Ctrl with a letter during a search
- **GIVEN** a search is open with the query `fi`
- **WHEN** the user presses Ctrl+`a`
- **THEN** the app receives `::fi` and then Ctrl+`a`
- **AND** the search is closed

#### Scenario: An arrow key during a search
- **GIVEN** a search is open with the query `fi`
- **WHEN** the user presses Left
- **THEN** the app receives `::fi` and then Left
- **AND** the search is closed

### Requirement: The trigger counts only at a boundary
The trigger SHALL open a search only at a boundary: at the start of the text, or after whitespace, an emoji, an opening bracket or an opening quote. The opening brackets and quotes are `( [ { " ' « „ “`. The engine SHALL use the surrounding text when the app sent it.

#### Scenario: A trigger after a word
- **GIVEN** the surrounding text before the cursor is `std`
- **WHEN** the user types `::`
- **THEN** the app receives `::` as typed
- **AND** no search opens

#### Scenario: A trigger after a space
- **GIVEN** the surrounding text before the cursor is `hello `
- **WHEN** the user types `::`
- **THEN** a search opens

#### Scenario: A trigger at the start of the text
- **GIVEN** the surrounding text is empty
- **WHEN** the user types `::`
- **THEN** a search opens

#### Scenario: A trigger after an emoji
- **GIVEN** the surrounding text before the cursor is `ok 🔥`
- **WHEN** the user types `::`
- **THEN** a search opens

#### Scenario: A trigger after an opening bracket
- **GIVEN** the surrounding text before the cursor is `(`
- **WHEN** the user types `::smile:`
- **THEN** the app receives `😄`

#### Scenario: A trigger after an opening quote
- **GIVEN** the surrounding text before the cursor is `„`
- **WHEN** the user types `::`
- **THEN** a search opens

### Requirement: The engine asks every app for surrounding text
The engine SHALL ask each input context for surrounding text when it gains focus, not only the first one.

#### Scenario: A second input context is asked
- **GIVEN** the engine runs and one input context already had focus
- **WHEN** a second input context gains focus
- **THEN** the second input context is asked for surrounding text

### Requirement: Without surrounding text the last typed character decides
When the app sent no surrounding text, the engine SHALL use the last character it passed through or committed since focus-in. When it knows no character, the engine SHALL count the cursor as a boundary.

#### Scenario: A trigger after a typed letter
- **GIVEN** the app sends no surrounding text
- **WHEN** the user types `d` and then `::`
- **THEN** the app receives `d::` as typed
- **AND** no search opens

#### Scenario: A trigger after a typed space
- **GIVEN** the app sends no surrounding text
- **WHEN** the user types `d`, a space and then `::`
- **THEN** a search opens

#### Scenario: A trigger with nothing known
- **GIVEN** the app sends no surrounding text and the user typed nothing since focus-in
- **WHEN** the user types `::`
- **THEN** a search opens

#### Scenario: Two emoji in a row
- **GIVEN** the app sends no surrounding text
- **WHEN** the user types `::fire:::rocket:`
- **THEN** the app receives `🔥🚀`

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

### Requirement: A trigger prefix is held
While the typed keys are a proper prefix of the trigger, the engine SHALL hold them in the preedit. When the next key press does not continue the trigger and is neither Backspace nor a modifier key alone, the engine SHALL commit the held text and handle the key as usual.

#### Scenario: The first colon is held
- **GIVEN** the cursor is at a boundary
- **WHEN** the user types `:`
- **THEN** the preedit shows `:`
- **AND** the engine commits nothing

#### Scenario: A letter after the first colon
- **GIVEN** the user typed `:` at a boundary
- **WHEN** the user types `a`
- **THEN** the app receives `:a`
- **AND** no search opens

### Requirement: Backspace removes a held trigger prefix
Backspace SHALL remove a held trigger prefix without passing through.

#### Scenario: Backspace after the first colon
- **GIVEN** the user typed `:` at a boundary
- **WHEN** the user presses Backspace
- **THEN** the preedit is empty
- **AND** the app receives neither `:` nor Backspace

### Requirement: The preedit shows the search
During a search the preedit SHALL show the trigger and the query, underlined.

#### Scenario: The preedit of a search
- **WHEN** the user types `::fi` at a boundary
- **THEN** the preedit shows `::fi`
- **AND** all four characters are underlined

### Requirement: Query characters
The query SHALL accept ASCII letters, digits, `_`, `+` and `-`. The search SHALL ignore the case of letters.

#### Scenario: A shortcode with a plus sign
- **WHEN** the user types `::+1` at a boundary
- **THEN** the candidate list shows 👍

#### Scenario: A shortcode with digits and underscores
- **WHEN** the user types `::1st_place` at a boundary
- **THEN** the candidate list shows 🥇

#### Scenario: Upper case letters
- **WHEN** the user types `::FIRE` at a boundary
- **THEN** the first candidate is 🔥

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

### Requirement: Arrow keys move the selection
During a search with a candidate list, Up and Down SHALL move the selection by one candidate. The selection SHALL stay within the list.

#### Scenario: Down selects the next candidate
- **GIVEN** a search is open with the query `fire`
- **WHEN** the user presses Down and then Enter
- **THEN** the app receives the second candidate's emoji

#### Scenario: Up at the first candidate
- **GIVEN** a search is open with the query `fire`
- **WHEN** the user presses Up
- **THEN** the first candidate stays selected
- **AND** Up does not reach the app

### Requirement: Page keys move by a page
During a search with a candidate list, Page Down and Page Up SHALL move the selection by one page of 9 candidates. The selection SHALL stay within the list.

#### Scenario: Page Down
- **GIVEN** a search is open with the query `fi`, which has more than 9 matches
- **WHEN** the user presses Page Down
- **THEN** the tenth candidate is selected

#### Scenario: Page Up after Page Down
- **GIVEN** a search is open with the query `fi`
- **WHEN** the user presses Page Down and then Page Up
- **THEN** the first candidate is selected

### Requirement: Enter and Tab commit the selected emoji
During a search with a candidate list, Enter and Tab SHALL commit the selected emoji and close the search. The key SHALL NOT reach the app.

#### Scenario: Enter commits
- **GIVEN** a search is open with the query `fire`
- **WHEN** the user presses Enter
- **THEN** the app receives 🔥 and no Enter
- **AND** the preedit is empty and the candidate list is hidden

#### Scenario: Tab commits
- **GIVEN** a search is open with the query `fire`
- **WHEN** the user presses Tab
- **THEN** the app receives 🔥 and no Tab

### Requirement: A click commits a candidate
A click on a candidate SHALL commit that candidate's emoji and close the search.

#### Scenario: A click on the first page
- **GIVEN** a search is open with the query `fire`
- **WHEN** the user clicks the third candidate
- **THEN** the app receives the third candidate's emoji

#### Scenario: A click on the second page
- **GIVEN** a search is open with the query `fi` and the user pressed Page Down
- **WHEN** the user clicks the first candidate shown
- **THEN** the app receives the tenth candidate's emoji

### Requirement: Backspace edits the query
Backspace SHALL remove the last character of the query. With an empty query, Backspace SHALL remove the trigger and close the search. Backspace SHALL NOT reach the app during a search.

#### Scenario: Backspace removes a query character
- **GIVEN** a search is open with the query `fir`
- **WHEN** the user presses Backspace
- **THEN** the preedit shows `::fi`

#### Scenario: Backspace with an empty query
- **GIVEN** a search is open with an empty query
- **WHEN** the user presses Backspace
- **THEN** the preedit is empty and the search is closed
- **AND** the app receives nothing

#### Scenario: Backspace to an empty query hides the list
- **GIVEN** a search is open with the query `f`
- **WHEN** the user presses Backspace
- **THEN** the preedit shows `::` and no candidate list is shown

### Requirement: Escape cancels the search
Esc SHALL close the search and commit the plain text. Esc SHALL NOT reach the app.

#### Scenario: Escape during a search
- **GIVEN** a search is open with the query `fi`
- **WHEN** the user presses Esc
- **THEN** the app receives `::fi` and no Esc

#### Scenario: Escape with an empty query
- **GIVEN** a search is open with an empty query
- **WHEN** the user presses Esc
- **THEN** the app receives `::`
- **AND** Esc does not reach the app

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

### Requirement: Enter and Tab without a candidate list
During a search with no candidate list, Enter and Tab SHALL close the search, commit the plain text and reach the app.

#### Scenario: Enter with an empty query
- **GIVEN** a search is open with an empty query
- **WHEN** the user presses Enter
- **THEN** the app receives `::` and then Enter

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

### Requirement: Held text is never lost
When the input context loses focus, is reset, or the engine is switched off, the held text SHALL reach the app exactly once. The next key SHALL then start from no held text.

#### Scenario: Focus moves away during a search
- **GIVEN** a search is open with the query `fi`
- **WHEN** the input context loses focus
- **THEN** the app receives `::fi` once

#### Scenario: The app resets the input context during a search
- **GIVEN** a search is open with the query `fi`
- **WHEN** the app resets the input context
- **THEN** the app receives `::fi` once

#### Scenario: The engine is switched off during a search
- **GIVEN** a search is open with the query `fi`
- **WHEN** the user switches to another input source
- **THEN** the app receives `::fi` once

#### Scenario: Focus moves away with a held dead key
- **GIVEN** the user pressed the dead acute key
- **WHEN** the input context loses focus
- **THEN** the app receives `´` once

#### Scenario: Typing after a reset
- **GIVEN** a search was open with the query `fi` and the app reset the input context
- **WHEN** the user types `a`
- **THEN** the key reaches the app
- **AND** no search is open

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
