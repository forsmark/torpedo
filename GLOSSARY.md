# Glossary

The words of this domain. They are binding in code, specs, issues and commit
messages. One name per thing: when a word is here, use it and no synonym.

Each term is a line `**Term**:` followed by its definition. `spec-checks claims`
reads these lines to find the specs a change could contradict.

<!-- Add terms when a change introduces them. The domain-modeling skill helps. -->

**Alias**:
A name the user adds to an emoji in the settings window, in addition to its shortcodes.

**Boundary**:
A place where the trigger counts: the start of the text, or right after whitespace.

**Candidate list**:
The list of matching emoji that GNOME Shell shows at the text cursor while a search is open.

**Component file**:
The XML file in `/usr/share/ibus/component/` that tells IBus how to start the engine and names the engine `torpedo`.

**Engine**:
The torpedo program that IBus starts and sends each key to. It decides whether a key passes through to the app.

**Held text**:
Text the user typed that the engine shows in the preedit and has not committed yet: a trigger prefix, or the trigger and the query.

**Plain text**:
The held text committed exactly as typed, with no emoji in place of it.

**Preedit**:
The underlined text that the app shows at the cursor while the engine holds it. It is not part of the app's text until the engine commits it.

**Query**:
The characters typed after the trigger during a search. The search matches them against shortcodes.

**Search**:
The state that starts when the user types the full trigger at a boundary. It ends when the engine commits an emoji or the plain text.

**Shortcode**:
A Slack-style emoji name, such as thumbsup or +1, from gemoji data.

**Surrounding text**:
The text around the cursor that the app sends to the engine. The engine reads the character before the cursor from it to find a boundary.

**Trigger**:
The key sequence, such as "::", that starts a search for an emoji name.
