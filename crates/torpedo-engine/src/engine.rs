//! The D-Bus objects IBus talks to: the factory, one engine object per
//! `CreateEngine`, and the service object that IBus destroys.
//!
//! Each engine object owns a `Picker` and turns its actions into IBus
//! signals, in order, before it answers the key.

use std::sync::atomic::{AtomicU32, Ordering};

use torpedo_core::picker::{Action, Outcome, PAGE_SIZE, Picker};
use zbus::object_server::SignalEmitter;
use zbus::zvariant::{OwnedObjectPath, Value};
use zbus::{fdo, interface};

use crate::ibus_types::{self, ORIENTATION_VERTICAL, PREEDIT_CLEAR, PREEDIT_COMMIT};

/// The engine name in the component file.
pub const ENGINE_NAME: &str = "torpedo";

/// Print a line to stderr when `TORPEDO_DEBUG` is set. The daemon shows
/// engine stderr only with `--verbose`.
pub fn debug(message: impl AsRef<str>) {
    if std::env::var_os("TORPEDO_DEBUG").is_some() {
        eprintln!("[torpedo] {}", message.as_ref());
    }
}

/// The text of an IBusText that arrives as a `v`.
fn ibus_text_string(value: &Value<'_>) -> Option<String> {
    match value {
        Value::Value(inner) => ibus_text_string(inner),
        Value::Structure(s) => match s.fields().get(2) {
            Some(Value::Str(text)) => Some(text.to_string()),
            _ => None,
        },
        _ => None,
    }
}

/// One input method engine, as IBus sees it.
pub struct Engine {
    picker: Picker,
}

impl Engine {
    pub fn new() -> Engine {
        Engine {
            picker: Picker::with_defaults(),
        }
    }

    async fn emit(em: &SignalEmitter<'_>, outcome: &Outcome) -> zbus::Result<()> {
        for action in &outcome.actions {
            match action {
                Action::Commit(text) => Self::commit_text(em, ibus_types::text(text)).await?,
                Action::ShowPreedit(text) => {
                    let cursor = text.chars().count() as u32;
                    Self::update_preedit_text(
                        em,
                        ibus_types::underlined(text),
                        cursor,
                        true,
                        PREEDIT_COMMIT,
                    )
                    .await?
                }
                Action::ClearPreedit => {
                    Self::update_preedit_text(em, ibus_types::text(""), 0, false, PREEDIT_CLEAR)
                        .await?
                }
                Action::ShowCandidates { items, cursor } => {
                    let table = ibus_types::lookup_table(
                        items,
                        PAGE_SIZE as u32,
                        *cursor as u32,
                        ORIENTATION_VERTICAL,
                    );
                    Self::update_lookup_table(em, table, true).await?
                }
                Action::HideCandidates => Self::hide_lookup_table(em).await?,
            }
        }
        Ok(())
    }

    async fn apply(em: &SignalEmitter<'_>, outcome: Outcome) -> fdo::Result<()> {
        Self::emit(em, &outcome)
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))
    }
}

impl Default for Engine {
    fn default() -> Self {
        Engine::new()
    }
}

#[interface(name = "org.freedesktop.IBus.Engine")]
impl Engine {
    async fn process_key_event(
        &mut self,
        keyval: u32,
        keycode: u32,
        state: u32,
        #[zbus(signal_emitter)] em: SignalEmitter<'_>,
    ) -> fdo::Result<bool> {
        let outcome = self.picker.key(keyval, state);
        debug(format!(
            "key {keyval:#x} code {keycode} state {state:#x} -> {:?} consumed {}",
            outcome.actions, outcome.consumed
        ));
        let consumed = outcome.consumed;
        Self::apply(&em, outcome).await?;
        Ok(consumed)
    }

    fn set_surrounding_text(&mut self, text: Value<'_>, cursor_pos: u32, anchor_pos: u32) {
        if let Some(text) = ibus_text_string(&text) {
            debug(format!(
                "surrounding {text:?} cursor {cursor_pos} anchor {anchor_pos}"
            ));
            self.picker.surrounding_text(&text, cursor_pos, anchor_pos);
        }
    }

    async fn focus_in(&mut self, #[zbus(signal_emitter)] em: SignalEmitter<'_>) -> fdo::Result<()> {
        debug("focus in");
        Self::apply(&em, self.picker.focus_in()).await
    }

    async fn focus_in_id(
        &mut self,
        _object_path: &str,
        _client: &str,
        #[zbus(signal_emitter)] em: SignalEmitter<'_>,
    ) -> fdo::Result<()> {
        Self::apply(&em, self.picker.focus_in()).await
    }

    async fn focus_out(
        &mut self,
        #[zbus(signal_emitter)] em: SignalEmitter<'_>,
    ) -> fdo::Result<()> {
        debug("focus out");
        Self::apply(&em, self.picker.drop_state()).await
    }

    async fn focus_out_id(
        &mut self,
        _object_path: &str,
        #[zbus(signal_emitter)] em: SignalEmitter<'_>,
    ) -> fdo::Result<()> {
        Self::apply(&em, self.picker.drop_state()).await
    }

    async fn reset(&mut self, #[zbus(signal_emitter)] em: SignalEmitter<'_>) -> fdo::Result<()> {
        debug("reset");
        Self::apply(&em, self.picker.drop_state()).await
    }

    fn enable(&self) {
        debug("enable");
    }

    async fn disable(&mut self, #[zbus(signal_emitter)] em: SignalEmitter<'_>) -> fdo::Result<()> {
        debug("disable");
        Self::apply(&em, self.picker.drop_state()).await
    }

    async fn candidate_clicked(
        &mut self,
        index: u32,
        _button: u32,
        _state: u32,
        #[zbus(signal_emitter)] em: SignalEmitter<'_>,
    ) -> fdo::Result<()> {
        debug(format!("candidate clicked {index}"));
        Self::apply(&em, self.picker.candidate_clicked(index)).await
    }

    async fn page_up(&mut self, #[zbus(signal_emitter)] em: SignalEmitter<'_>) -> fdo::Result<()> {
        Self::apply(&em, self.picker.move_selection(-(PAGE_SIZE as isize))).await
    }

    async fn page_down(
        &mut self,
        #[zbus(signal_emitter)] em: SignalEmitter<'_>,
    ) -> fdo::Result<()> {
        Self::apply(&em, self.picker.move_selection(PAGE_SIZE as isize)).await
    }

    async fn cursor_up(
        &mut self,
        #[zbus(signal_emitter)] em: SignalEmitter<'_>,
    ) -> fdo::Result<()> {
        Self::apply(&em, self.picker.move_selection(-1)).await
    }

    async fn cursor_down(
        &mut self,
        #[zbus(signal_emitter)] em: SignalEmitter<'_>,
    ) -> fdo::Result<()> {
        Self::apply(&em, self.picker.move_selection(1)).await
    }

    fn set_cursor_location(&self, _x: i32, _y: i32, _w: i32, _h: i32) {}

    fn set_capabilities(&self, caps: u32) {
        debug(format!("capabilities {caps:#x}"));
    }

    fn property_activate(&self, _name: &str, _state: u32) {}

    fn property_show(&self, _name: &str) {}

    fn property_hide(&self, _name: &str) {}

    // The daemon reads these as plain booleans, though introspection in
    // libibus says `(b)`. ActiveSurroundingText must be true, or the daemon
    // asks only the first input context for surrounding text.
    #[zbus(property, name = "FocusId")]
    fn focus_id(&self) -> bool {
        false
    }

    #[zbus(property, name = "ActiveSurroundingText")]
    fn active_surrounding_text(&self) -> bool {
        true
    }

    #[zbus(property, name = "ContentType")]
    fn content_type(&self) -> (u32, u32) {
        (0, 0)
    }

    #[zbus(property, name = "ContentType")]
    fn set_content_type(&mut self, _value: (u32, u32)) {}

    #[zbus(signal)]
    async fn commit_text(em: &SignalEmitter<'_>, text: Value<'_>) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn update_preedit_text(
        em: &SignalEmitter<'_>,
        text: Value<'_>,
        cursor_pos: u32,
        visible: bool,
        mode: u32,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn update_lookup_table(
        em: &SignalEmitter<'_>,
        table: Value<'_>,
        visible: bool,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn hide_lookup_table(em: &SignalEmitter<'_>) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn forward_key_event(
        em: &SignalEmitter<'_>,
        keyval: u32,
        keycode: u32,
        state: u32,
    ) -> zbus::Result<()>;
}

/// `org.freedesktop.IBus.Service` on each engine object. The daemon calls
/// Destroy when it drops the engine, for example when the user switches away.
pub struct Service;

#[interface(name = "org.freedesktop.IBus.Service")]
impl Service {
    async fn destroy(
        &self,
        #[zbus(header)] header: zbus::message::Header<'_>,
        #[zbus(object_server)] server: &zbus::ObjectServer,
    ) {
        let Some(path) = header.path().map(|p| p.to_owned()) else {
            return;
        };
        debug(format!("destroy {path}"));
        // Remove later: this call runs on the very object it removes.
        let server = server.clone();
        tokio::spawn(async move {
            let _ = server.remove::<Engine, _>(&path).await;
            let _ = server.remove::<Service, _>(&path).await;
        });
    }
}

/// `org.freedesktop.IBus.Factory` at `/org/freedesktop/IBus/Factory`.
#[derive(Default)]
pub struct Factory {
    created: AtomicU32,
}

#[interface(name = "org.freedesktop.IBus.Factory")]
impl Factory {
    async fn create_engine(
        &self,
        name: &str,
        #[zbus(object_server)] server: &zbus::ObjectServer,
    ) -> fdo::Result<OwnedObjectPath> {
        debug(format!("create engine {name}"));
        if name != ENGINE_NAME {
            return Err(fdo::Error::Failed(format!("unknown engine {name}")));
        }
        let n = self.created.fetch_add(1, Ordering::SeqCst) + 1;
        let path = format!("/org/freedesktop/IBus/Engine/{n}");
        let failed = |e: zbus::Error| fdo::Error::Failed(e.to_string());
        server
            .at(path.as_str(), Engine::new())
            .await
            .map_err(failed)?;
        server.at(path.as_str(), Service).await.map_err(failed)?;
        OwnedObjectPath::try_from(path).map_err(|e| fdo::Error::Failed(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_text_of_an_ibus_text() {
        let v = Value::new(ibus_types::text("hello "));
        assert_eq!(ibus_text_string(&v).as_deref(), Some("hello "));
    }
}
