# Spec Delta

## Purpose

The inline picker lets the user type a trigger and a shortcode in any text field and insert the matching emoji, while all other typing reaches the app unchanged.

## ADDED Requirements

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

### Requirement: Dead keys pass through
A dead key, such as the acute accent on the Danish layout, SHALL pass through.

#### Scenario: A dead key with nothing held
- **GIVEN** no text is held
- **WHEN** the user presses the dead acute key
- **THEN** the key reaches the app

#### Scenario: A dead key while the first colon is held
- **GIVEN** the user typed `:` at a boundary
- **WHEN** the user presses the dead acute key
- **THEN** the app receives `:` and then the dead acute key

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
The trigger SHALL open a search only at a boundary: at the start of the text or after whitespace. The engine SHALL use the surrounding text when the app sent it.

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

### Requirement: The engine asks every app for surrounding text
The engine SHALL ask each input context for surrounding text when it gains focus, not only the first one.

#### Scenario: A second input context is asked
- **GIVEN** the engine runs and one input context already had focus
- **WHEN** a second input context gains focus
- **THEN** the second input context is asked for surrounding text

### Requirement: Without surrounding text the last typed character decides
When the app sent no surrounding text, the engine SHALL use the last character it saw since focus-in. When it knows no character, the engine SHALL count the cursor as a boundary.

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

### Requirement: The trigger is two colons
The trigger SHALL be `::`.

#### Scenario: Two colons open a search
- **GIVEN** the cursor is at a boundary
- **WHEN** the user types `::`
- **THEN** a search opens with an empty query
- **AND** the preedit shows `::`

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
The candidate list SHALL appear at the cursor when the query has at least one character and matches. It SHALL show 9 candidates per page with no labels. Each candidate SHALL show the emoji and its best matching shortcode.

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

### Requirement: Each emoji appears once
An emoji with several matching shortcodes SHALL appear once in the candidate list, under its best ranked shortcode.

#### Scenario: An emoji with two matching shortcodes
- **WHEN** the user types `::fist` at a boundary
- **THEN** ✊ appears once, as `✊ fist`

### Requirement: Ranking
The candidate list SHALL rank an exact shortcode first, then a shortcode that starts with the query, then a shortcode with a `_`-separated word that starts with the query, then a shortcode that holds the query. Ties SHALL keep gemoji's order.

#### Scenario: Exact before prefix before word prefix
- **WHEN** the user types `::fire` at a boundary
- **THEN** the candidates are 🔥 fire, then the shortcodes that start with `fire` such as fire_engine, then the shortcodes with a word that starts with `fire` such as heart_on_fire

#### Scenario: Prefix before substring
- **WHEN** the user types `::ire` at a boundary
- **THEN** 🇮🇪 ireland comes before 😫 tired_face

#### Scenario: Ties keep gemoji's order
- **WHEN** the user types `::thumbs` at a boundary
- **THEN** 👍 thumbsup comes before 👎 thumbsdown

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
When the query is an exact shortcode, `:` SHALL commit that shortcode's emoji and close the search.

#### Scenario: Slack style
- **WHEN** the user types `::fire:` at a boundary
- **THEN** the app receives 🔥

#### Scenario: A colon after a partial shortcode
- **WHEN** the user types `::fir:` at a boundary
- **THEN** the app receives `::fir:`

### Requirement: Another character closes the search
A character that is not a query character, other than the `:` after an exact shortcode, SHALL close the search, commit the plain text, and then be handled as usual.

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
When the query stops matching any shortcode, the engine SHALL commit the plain text and close the search.

#### Scenario: No shortcode matches
- **GIVEN** a search is open with the query `fire`
- **WHEN** the user types `q`
- **THEN** the app receives `::fireq`
- **AND** the search is closed

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

#### Scenario: Typing after a reset
- **GIVEN** a search was open with the query `fi` and the app reset the input context
- **WHEN** the user types `a`
- **THEN** the key reaches the app
- **AND** no search is open
