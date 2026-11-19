# skin-tone Specification

## Purpose
The skin tone lets the user pick one tone for every emoji that has toned forms. The engine shows and commits those emoji in that tone, and never changes an alias text.

## Requirements

### Requirement: The skin tone applies to emoji with toned forms
When the config file sets a skin tone other than `none`, the candidate list SHALL show each emoji that gemoji marks with `skin_tones` in that tone. The engine SHALL commit such an emoji in that tone. Other emoji SHALL stay as they are. With the skin tone `none`, the engine SHALL change no emoji.

#### Scenario: A candidate in a tone
- **GIVEN** the config file sets `skin_tone = "medium"`
- **WHEN** the user types `::wave` at a boundary
- **THEN** the first candidate reads `👋🏽 wave`

#### Scenario: Enter commits the tone
- **GIVEN** the config file sets `skin_tone = "medium"`
- **WHEN** the user types `::wave` at a boundary and presses Enter
- **THEN** the app receives 👋🏽

#### Scenario: The closing colon commits the tone
- **GIVEN** the config file sets `skin_tone = "dark"`
- **WHEN** the user types `::+1:` at a boundary
- **THEN** the app receives 👍🏿

#### Scenario: An emoji with no toned form
- **GIVEN** the config file sets `skin_tone = "medium"`
- **WHEN** the user types `::fire:` at a boundary
- **THEN** the app receives 🔥

#### Scenario: No skin tone
- **GIVEN** the config file sets `skin_tone = "none"`
- **WHEN** the user types `::wave:` at a boundary
- **THEN** the app receives 👋 with no modifier

### Requirement: The toned form of an emoji
The engine SHALL put the modifier of the skin tone right after each Emoji_Modifier_Base character of the emoji. It SHALL remove a VS16 (U+FE0F) that directly follows such a character. A modifier base between two ZWJs (U+200D) SHALL stay without a modifier. Each toned form SHALL be a sequence that Unicode recommends.

#### Scenario: A hand in the light tone
- **GIVEN** the config file sets `skin_tone = "light"`
- **WHEN** the user types `::raised_back_of_hand:` at a boundary
- **THEN** the app receives 🤚🏻, which is U+1F91A U+1F3FB

#### Scenario: The VS16 after a modifier base goes
- **GIVEN** the config file sets `skin_tone = "medium"`
- **WHEN** the user types `::v:` at a boundary
- **THEN** the app receives U+270C U+1F3FD, with no U+FE0F

#### Scenario: A person with a sign
- **GIVEN** the config file sets `skin_tone = "medium-dark"`
- **WHEN** the user types `::golfing_woman:` at a boundary
- **THEN** the app receives U+1F3CC U+1F3FE U+200D U+2640 U+FE0F

#### Scenario: Two people get the same tone
- **GIVEN** the config file sets `skin_tone = "light"`
- **WHEN** the user types `::couple_with_heart_woman_man:` at a boundary
- **THEN** the app receives U+1F469 U+1F3FB U+200D U+2764 U+FE0F U+200D U+1F468 U+1F3FB

#### Scenario: The handshake between two people stays bare
- **GIVEN** the config file sets `skin_tone = "medium-light"`
- **WHEN** the user types `::people_holding_hands:` at a boundary
- **THEN** the app receives U+1F9D1 U+1F3FC U+200D U+1F91D U+200D U+1F9D1 U+1F3FC

#### Scenario: Every toned form is a sequence Unicode recommends
- **GIVEN** the IBus emoji dictionary `/usr/share/ibus/dicts/emoji-en.dict`
- **WHEN** the toned forms of every gemoji emoji with `skin_tones` are built in each of the five tones
- **THEN** each toned form is a sequence of that dictionary

### Requirement: Alias texts never take a skin tone
The engine SHALL commit an alias text exactly as the config file writes it, whatever the skin tone. A candidate shown under an alias name SHALL show the alias text as the config file writes it.

#### Scenario: An alias of a hand
- **GIVEN** the config file sets `skin_tone = "medium"` and has the alias `hi = "👋"`
- **WHEN** the user types `::hi:` at a boundary
- **THEN** the app receives 👋 with no modifier

#### Scenario: An alias and a shortcode of the same emoji
- **GIVEN** the config file sets `skin_tone = "medium"` and has the alias `wave_hi = "👋"`
- **WHEN** the user types `::wave` at a boundary
- **THEN** the first candidate reads `👋🏽 wave`
- **AND** no candidate reads `👋 wave_hi`

#### Scenario: The alias name as the best match
- **GIVEN** the config file sets `skin_tone = "medium"` and has the alias `wave_hi = "👋"`
- **WHEN** the user types `::wave_h` at a boundary
- **THEN** the first candidate reads `👋 wave_hi`

### Requirement: Usage records the emoji without its skin tone
Usage SHALL record an emoji without its skin tone, so a change of the skin tone keeps the ranking.

#### Scenario: A toned commit records the bare emoji
- **GIVEN** the config file sets `skin_tone = "medium"` and no usage file exists
- **WHEN** the user types `::wave:` at a boundary
- **THEN** the usage file holds 👋
- **AND** it holds no 👋🏽

#### Scenario: A new skin tone keeps the ranking
- **GIVEN** a usage file with 👎 at the usage score 3, last committed now, and the config file sets `skin_tone = "dark"`
- **WHEN** the user types `::thumbs` at a boundary
- **THEN** the candidates are `👎🏿 thumbsdown` and `👍🏿 thumbsup`, in that order
