# The IBus protocol as torpedo uses it

Torpedo speaks the IBus D-Bus protocol directly with zbus. It does not link libibus. A spike against IBus 1.5.34 found the facts below. The source paths refer to the IBus source tree.

## Component path and registry cache

- The daemon loads component files from `/usr/share/ibus/component`. `IBUS_COMPONENT_PATH` replaces that folder. It is a colon-separated list (`src/ibusregistry.c`, `ibus_registry_load()`). A custom path must list `/usr/share/ibus/component` itself, or the `xkb:*` engines disappear.
- There is no per-user component folder. The code for one is under `#if 0`.
- With `--cache=auto`, the default, the daemon loads `/var/cache/ibus/bus/registry` and then `$XDG_CACHE_HOME/ibus/bus/registry`. It rebuilds the cache when a watched folder or a component file changed. The cache does not record `IBUS_COMPONENT_PATH`. `--cache=refresh` always rebuilds.
- `ibus write-cache` does not talk to the daemon. It loads the components with its own environment and saves the user cache.
- A running daemon never reloads its registry. So a new component file needs `ibus restart`.

## Address lookup

libibus finds the daemon in this order (`src/ibusshare.c`, `ibus_get_address()` and `ibus_get_socket_path()`):

1. `$IBUS_ADDRESS`.
2. The file named by `$IBUS_ADDRESS_FILE`.
3. `$XDG_CONFIG_HOME/ibus/bus/<machine-id>-<host>-<display>`. With `WAYLAND_DISPLAY` set, host is `unix` and display is its value, as in `<id>-unix-wayland-0`. Otherwise `DISPLAY` splits into host and display number at `:`, a screen number after `.` is dropped, and an empty host becomes `unix`. The machine id comes from `/var/lib/dbus/machine-id`, else `/etc/machine-id`.

The file holds lines `IBUS_ADDRESS=unix:path=...,guid=...` and `IBUS_DAEMON_PID=N`, and `#` comments. A file whose PID is not alive is ignored.

The daemon does not pass `IBUS_ADDRESS` to the engines it starts. An engine inherits `IBUS_ADDRESS_FILE` only when the daemon had it. So a real engine must read the address file.

## GVariant shapes

Each IBus object (an IBusSerializable) is a struct that starts with `(s type_name, a{sv} attachments)` and travels wrapped in `v`.

| Type | Signature | Fields after the header |
|---|---|---|
| IBusAttribute | `(sa{sv}uuuu)` | type, value, start, end (in characters) |
| IBusAttrList | `(sa{sv}av)` | attributes, each in `v` |
| IBusText | `(sa{sv}sv)` | text, attribute list in `v` |
| IBusLookupTable | `(sa{sv}uubbiavav)` | page_size, cursor_pos, cursor_visible, round, orientation, candidates (`av` of IBusText), labels (`av` of IBusText) |

Attribute type 1 is UNDERLINE, 2 FOREGROUND and 3 BACKGROUND. Underline value 1 is SINGLE. Orientation 0 is horizontal, 1 vertical and 2 the system default.

Pitfall: in zvariant, a `Value` used as a tuple field or as an element of a `Vec<Value>` is already encoded as `v`. Never wrap these in `Value::new()`, which gives `v<v<..>>`. libibus tolerates one extra level at the top, but the nested IBusAttrList then fails with a GLib-CRITICAL. The correct nesting of an IBusText is `(s,a{sv},s,v<(s,a{sv},[v<(s,a{sv},u,u,u,u)>])>)`.

## The D-Bus surface

- ibus-daemon is a D-Bus bus itself. It answers Hello and RequestName and uses EXTERNAL auth for the same user. zbus connects with `connection::Builder::address("unix:path=...,guid=...")`.
- The daemon starts the component's `<exec>` when the engine is first selected.
- Export `/org/freedesktop/IBus/Factory` (`org.freedesktop.IBus.Factory`) before requesting the bus name of the component. The daemon calls `CreateEngine(s) -> o` as soon as the name appears. Export the engine object, such as `/org/freedesktop/IBus/Engine/1`, before replying. `ObjectServer::at()` works inside the handler.
- Calls the daemon makes on the engine: Properties GetAll, Get and Set, FocusIn, FocusInId(s, s), FocusOut, FocusOutId(s), Enable, Disable, SetCapabilities(u), SetSurroundingText(vuu), SetCursorLocation(iiii), Properties.Set of ContentType (uu), ProcessKeyEvent(uuu) -> b, Reset, CandidateClicked(uuu), PageUp, PageDown, CursorUp, CursorDown, and `org.freedesktop.IBus.Service.Destroy` on the engine object when the user switches away. Switching back calls CreateEngine again in the same process.
- When the daemon exits, the connection closes. The engine must exit by itself.
- The properties `FocusId` and `ActiveSurroundingText` are plain `b`. Introspection says `(b)`, but the daemon reads them with `g_variant_get_boolean`. The daemon reads them once per engine name. With FocusId true, it calls FocusInId and FocusOutId.
- A missing method gives an UnknownMethod error, which the daemon ignores. Only CreateEngine and ProcessKeyEvent are strictly needed. Without the properties, no surrounding text arrives.
- Signals the daemon handles: CommitText(v), UpdatePreeditText(v, u cursor, b visible, u mode), UpdateAuxiliaryText(v, b), UpdateLookupTable(v, b), Show and Hide of PreeditText, AuxiliaryText and LookupTable, PageUp, PageDown, CursorUp and CursorDown LookupTable, RequireSurroundingText, ForwardKeyEvent(uuu), DeleteSurroundingText(iu), RegisterProperties(v), UpdateProperty(v), PanelExtension(v).
- With use-global-engine on, which is the default, the daemon keeps a "fake" input context with caps 0x29. FocusIn arrives from it after a client's focus-out.

## Preedit mode and held text

UpdatePreeditText's fourth argument is the mode: 0 is `IBUS_ENGINE_PREEDIT_CLEAR` and 1 is `IBUS_ENGINE_PREEDIT_COMMIT` (`src/ibustypes.h`). With mode COMMIT and a visible preedit, the daemon commits the preedit itself on focus-out, reset, disable and engine switch (`bus/inputcontext.c`, `bus_input_context_clear_preedit_text`). An engine that uses COMMIT must then only drop its state on those calls. When a client sets `client_commit_preedit`, the daemon drops the preedit and the client commits it.

## Clicks come from the panel

Only a panel can send a click. The daemon forwards the `CandidateClicked` signal of the owner of `org.freedesktop.IBus.Panel` to the focused input context (`bus/panelproxy.c`, and `bus/ibusimpl.c` watches the name). The index is within the shown page, as `src/ibusenginesimple.c` reads it. In the desktop, GNOME Shell is the panel.

## Key events and order

- Keyvals are never remapped from keycodes (`use_sys_layout` is true). `<layout>dk</layout>` in the component file causes no remap in the daemon. The compositor applies the layout.
- Modifier bits: Shift 1<<0, Lock 1<<1, Control 1<<2, Mod1 (Alt) 1<<3, Mod2 (Num Lock) 1<<4, Mod4 1<<6, Mod5 (AltGr) 1<<7, Super 1<<26, Hyper 1<<27, Meta 1<<28, Release 1<<30.
- A client in sync mode sets `EffectivePostProcessKeyEvent`. The daemon then queues the signals that arrive while a key is in flight and hands them over with the reply (`bus_input_context_make_post_process_key_event`). For an async client, the CommitText signal and the ProcessKeyEvent reply are two messages on one connection.

### Measured order (add-emoji-engine, task 4.1)

The engine commits held text and then returns false for a key without a character. `e2e/harness/order.test.mjs` holds `:` and then sends Ctrl+a or Left with `process_key_event_async`, and records the arrival of each signal and of the reply in the client.

On 2026-11-19, with IBus 1.5.34 and the release build, the CommitText signal arrived before the ProcessKeyEvent reply in 600 of 600 runs for Ctrl+a and in 600 of 600 runs for Left (three runs of 200 each). The engine emits the signal before it sends its reply, and the daemon forwards both in that order on its connection to the client.

So torpedo keeps "commit, then return false" for keys without a character, and does not use ForwardKeyEvent. A character key that ends held text never depends on this order: torpedo commits the held text and the character together. The order between the two paths inside GNOME Shell is not measured.

## Client capabilities and surrounding text

- Capability bits: PREEDIT_TEXT 1, AUXILIARY_TEXT 2, LOOKUP_TABLE 4, FOCUS 8, PROPERTY 16, SURROUNDING_TEXT 32, OSK 64, SYNC_PROCESS_KEY 128.
- With the LOOKUP_TABLE capability, the client gets `update-lookup-table(table, visible)` and no separate show signal.
- A client sends surrounding text only after RequireSurroundingText. The daemon emits that once per engine name on the first `Get FocusId`, and on every FocusIn and Enable when `ActiveSurroundingText` is true. So torpedo reports `ActiveSurroundingText = true`, or only the first input context would send surrounding text.

## A private daemon for tests

`e2e/harness/` runs this. The command it uses:

```sh
/usr/bin/ibus-daemon --address=unix:path=$RUN/bus --cache=none --panel=disable \
  --emoji-extension=disable --config=disable --single --verbose
```

with `IBUS_COMPONENT_PATH`, `IBUS_ADDRESS_FILE`, `XDG_CONFIG_HOME`, `XDG_CACHE_HOME` and `XDG_DATA_HOME` in a temp folder, `GSETTINGS_BACKEND=memory` and `GIO_USE_VFS=local`, inside `dbus-run-session` with `e2e/harness/session.conf`.

- `--xim` defaults off. Never pass `-r` or `--replace`, which calls Exit on whatever daemon the address lookup finds. `--verbose` lets engine stderr through.
- A Unix socket path is limited to 107 bytes.
- `IBusInputContext.set_engine()` is refused while use-global-engine is on. Use `bus.set_global_engine("torpedo")`.
- A commit arrives only as a signal. Pump the GLib main context until it arrives.

## Safety traps

1. A private daemon overwrites the live address file unless `IBUS_ADDRESS_FILE` is set (`bus/server.c` calls `ibus_write_address` always). Always set `IBUS_ADDRESS_FILE` and a private `XDG_CONFIG_HOME`.
2. The daemon also owns `org.freedesktop.IBus` on the session bus and starts gvfs and ibus-portal through D-Bus activation. Run it inside `dbus-run-session` with a `session.conf` that lists no service folders, and set `GIO_USE_VFS=local`.
3. Stop each private daemon with SIGTERM and wait for it before its D-Bus session ends. A daemon that dies with its bus makes Ubuntu show a crash dialog on the desktop.
