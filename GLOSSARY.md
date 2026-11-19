# Glossary

The words of this domain. They are binding in code, specs, issues and commit
messages. One name per thing: when a word is here, use it and no synonym.

Each term is a line `**Term**:` followed by its definition. `spec-checks claims`
reads these lines to find the specs a change could contradict.

<!-- Add terms when a change introduces them. The domain-modeling skill helps. -->

**Alias**:
A name the user gives to an emoji or a short text in the config file, usually through the settings window. The search matches it like a shortcode.

**Boundary**:
A place where the trigger counts: the start of the text, or right after whitespace, an emoji, an opening bracket or an opening quote.

**Candidate list**:
The list of matching emoji that GNOME Shell shows at the text cursor while a search is open.

**Config file**:
The TOML file `$XDG_CONFIG_HOME/torpedo/config.toml` that holds the user's trigger and aliases. The engine reads it, and the settings window writes it.

**Component file**:
The XML file in `/usr/share/ibus/component/` that tells IBus how to start the engine and names the engine `torpedo`.

**Dead key**:
A key that types no character by itself but puts an accent on the next one, such as the acute key on the Danish layout. The engine composes it with the next character.

**Engine**:
The torpedo program that IBus starts and sends each key to. It decides whether a key passes through to the app.

**Held text**:
Text the user typed that the engine shows in the preedit and has not committed yet: a trigger prefix, the trigger and the query, or the accent of a dead key.

**Plain text**:
The held text committed exactly as typed, with no emoji in place of it.

**Preedit**:
The underlined text that the app shows at the cursor while the engine holds it. It is not part of the app's text until the engine commits it.

**Query**:
The characters typed after the trigger during a search. The search matches them against shortcodes and aliases.

**Search**:
The state that starts when the user types the full trigger at a boundary. It ends when the engine commits an emoji or the plain text, or when Backspace removes the trigger.

**Settings window**:
The `torpedo-settings` program, where the user edits the trigger and the aliases.

**Shortcode**:
A Slack-style emoji name, such as thumbsup or +1, from gemoji data.

**Surrounding text**:
The text around the cursor that the app sends to the engine. The engine reads the character before the cursor from it to find a boundary.

**Trigger**:
The one to three characters that start a search for an emoji name. It is "::" unless the config file sets another.
