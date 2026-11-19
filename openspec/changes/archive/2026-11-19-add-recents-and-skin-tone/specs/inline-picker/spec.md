# Spec Delta

## MODIFIED Requirements

### Requirement: Ranking
The candidate list SHALL rank an exact shortcode or alias first, then a shortcode or alias that starts with the query, then one with a `_`-separated word that starts with the query, then one that holds the query. Among the candidates of equal rank that have no usage, aliases SHALL come first in the order of their names, and shortcodes SHALL keep gemoji's order. *Used emoji come first within a rank* places the candidates that have usage.

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

## ADDED Requirements

### Requirement: Used emoji come first within a rank
Within each rank, the candidates whose text has usage SHALL come before the others, highest usage score first. Usage SHALL NOT move a candidate into a better rank. The other scenarios of this spec start with no usage.

#### Scenario: Two commits move an emoji to the front of its rank
- **GIVEN** no usage file exists
- **WHEN** the user types `::thumbsdown:` twice and then `::thumbs`
- **THEN** the candidates are `👎 thumbsdown` and `👍 thumbsup`, in that order

#### Scenario: A higher usage score comes first
- **GIVEN** a usage file with 👍 at the usage score 1 and 👎 at the usage score 3, both last committed now
- **WHEN** the user types `::thumbs` at a boundary
- **THEN** the first candidate reads `👎 thumbsdown`

#### Scenario: An unused exact match beats a used prefix match
- **GIVEN** no usage file exists
- **WHEN** the user types `::fire_extinguisher:` twice and then `::fire`
- **THEN** the first candidate reads `🔥 fire`
- **AND** the second candidate reads `🧯 fire_extinguisher`

#### Scenario: A used alias text comes first among aliases
- **GIVEN** the config file has the aliases `thumbsboth = "👍👎"` and `thumbsall = "🙌"`, and no usage file exists
- **WHEN** the user types `::thumbsboth:` and then `::thumbs`
- **THEN** the candidates are `👍👎 thumbsboth`, `🙌 thumbsall`, `👍 thumbsup` and `👎 thumbsdown`, in that order
