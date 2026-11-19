//! The settings window: the trigger and the aliases.
//!
//! Each good edit saves at once, as libadwaita preference windows do. A bad
//! value shows its problem under its field, and nothing is written while any
//! value is bad. A config file that does not parse shows a banner, and the
//! fields are read-only, because saving would drop what the user wrote.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::rc::Rc;

use adw::prelude::*;
use gtk::glib;
use torpedo_core::config::{self, Alias, ConfigDocument, DEFAULT_TRIGGER, LoadError};

/// The application id. A second start shows the open window.
pub const APP_ID: &str = "io.github.forsmark.Torpedo.Settings";

/// The problems of the values in the window, per field.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Checked {
    pub trigger: Option<String>,
    /// One entry per row, in order.
    pub rows: Vec<Option<String>>,
    /// The aliases to save, from the rows that are not blank.
    pub aliases: Vec<Alias>,
    /// A row that still has the focus is not good yet. It shows no problem,
    /// and nothing is saved until the focus leaves it.
    pub pending: bool,
}

impl Checked {
    /// No field shows a problem.
    pub fn is_good(&self) -> bool {
        self.trigger.is_none() && self.rows.iter().all(Option::is_none)
    }

    /// The values may be saved.
    pub fn can_save(&self) -> bool {
        self.is_good() && !self.pending
    }
}

/// One alias row of the window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowValues {
    pub name: String,
    pub text: String,
    /// The focus is in the row, so the user may still be typing.
    pub editing: bool,
}

/// Check the trigger and the alias rows with the rules of the config file. A
/// row with an empty name and an empty text is blank, and is skipped.
pub fn check_values(trigger: &str, rows: &[RowValues]) -> Checked {
    let mut out = Checked {
        trigger: config::check_trigger(trigger).err(),
        ..Checked::default()
    };
    let mut seen: Vec<String> = Vec::new();
    for RowValues {
        name,
        text,
        editing,
    } in rows
    {
        if name.is_empty() && text.is_empty() {
            out.rows.push(None);
            continue;
        }
        let problem = config::check_alias_name(name)
            .and_then(|()| config::check_alias_text(text))
            .err()
            .or_else(|| {
                seen.contains(&name.to_lowercase())
                    .then(|| "another alias has this name".to_string())
            });
        seen.push(name.to_lowercase());
        if problem.is_none() {
            out.aliases.push(Alias {
                name: name.clone(),
                text: text.clone(),
            });
        }
        if problem.is_some() && *editing {
            out.pending = true;
            out.rows.push(None);
        } else {
            out.rows.push(problem);
        }
    }
    out
}

struct AliasRow {
    root: gtk::Box,
    name: gtk::Entry,
    text: gtk::Entry,
    error: gtk::Label,
}

struct Ui {
    path: PathBuf,
    trigger: adw::EntryRow,
    trigger_error: gtk::Label,
    list: gtk::ListBox,
    rows: Vec<AliasRow>,
    toasts: adw::ToastOverlay,
    banner: adw::Banner,
    /// The values last read or written, so an unchanged field saves nothing.
    saved: Option<(String, Vec<Alias>)>,
}

type Shared = Rc<RefCell<Ui>>;

fn error_label() -> gtk::Label {
    let l = gtk::Label::builder()
        .xalign(0.0)
        .wrap(true)
        .visible(false)
        .margin_start(12)
        .margin_end(12)
        .margin_bottom(6)
        .build();
    l.add_css_class("error");
    l.add_css_class("caption");
    l
}

fn show_error(label: &gtk::Label, problem: &Option<String>) {
    match problem {
        Some(p) => {
            label.set_text(p);
            label.set_visible(true);
        }
        None => label.set_visible(false),
    }
}

/// Save after the current event, so that no callback runs inside another.
fn save_soon(ui: &Shared) {
    let ui = ui.clone();
    glib::idle_add_local_once(move || save(&ui));
}

fn save(ui: &Shared) {
    let Ok(mut ui) = ui.try_borrow_mut() else {
        return;
    };
    let rows: Vec<RowValues> = ui
        .rows
        .iter()
        .map(|r| RowValues {
            name: r.name.text().to_string(),
            text: r.text.text().to_string(),
            editing: r.root.state_flags().contains(gtk::StateFlags::FOCUS_WITHIN),
        })
        .collect();
    let trigger = ui.trigger.text().to_string();
    let checked = check_values(&trigger, &rows);
    show_error(&ui.trigger_error, &checked.trigger);
    if checked.trigger.is_some() {
        ui.trigger.add_css_class("error");
    } else {
        ui.trigger.remove_css_class("error");
    }
    for (row, problem) in ui.rows.iter().zip(&checked.rows) {
        show_error(&row.error, problem);
        for e in [&row.name, &row.text] {
            if problem.is_some() {
                e.add_css_class("error");
            } else {
                e.remove_css_class("error");
            }
        }
    }
    if !checked.can_save() {
        return;
    }
    let values = (trigger.clone(), checked.aliases.clone());
    if ui.saved.as_ref() == Some(&values) {
        return;
    }
    // Start from the file as it is now, so a hand edit of other keys or
    // comments since the window opened is kept.
    let base = match ConfigDocument::load(&ui.path) {
        Ok(d) => d,
        Err(e) => {
            ui.banner
                .set_title(&format!("Not saved. The config file has a problem: {e}"));
            ui.banner.set_revealed(true);
            return;
        }
    };
    match base.with_values(&trigger, &checked.aliases).save(&ui.path) {
        Ok(()) => {
            ui.banner.set_revealed(false);
            ui.toasts.add_toast(adw::Toast::new("Saved"));
            ui.saved = Some(values);
        }
        Err(e) => {
            ui.banner.set_title(&format!("Not saved: {e}"));
            ui.banner.set_revealed(true);
        }
    }
}

fn add_row(ui: &Shared, name: &str, text: &str) -> gtk::Entry {
    let name_entry = gtk::Entry::builder()
        .text(name)
        .placeholder_text("name")
        .width_chars(12)
        .tooltip_text("Letters, digits, _, + and -")
        .build();
    let text_entry = gtk::Entry::builder()
        .text(text)
        .placeholder_text("emoji or text")
        .hexpand(true)
        .tooltip_text("Up to 16 characters")
        .build();
    let delete = gtk::Button::builder()
        .icon_name("user-trash-symbolic")
        .tooltip_text("Delete this alias")
        .valign(gtk::Align::Center)
        .build();
    delete.add_css_class("flat");
    let line = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(6)
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(12)
        .margin_end(6)
        .build();
    line.append(&name_entry);
    line.append(&text_entry);
    line.append(&delete);
    let error = error_label();
    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    root.append(&line);
    root.append(&error);

    for entry in [&name_entry, &text_entry] {
        let u = ui.clone();
        entry.connect_activate(move |_| save_soon(&u));
        let focus = gtk::EventControllerFocus::new();
        let u = ui.clone();
        focus.connect_leave(move |_| save_soon(&u));
        entry.add_controller(focus);
    }
    let u = ui.clone();
    let r = root.clone();
    delete.connect_clicked(move |_| {
        {
            let mut ui = u.borrow_mut();
            ui.list.remove(&r);
            ui.rows.retain(|row| row.root != r);
        }
        save_soon(&u);
    });

    let mut state = ui.borrow_mut();
    state.list.append(&root);
    state.rows.push(AliasRow {
        root,
        name: name_entry.clone(),
        text: text_entry,
        error,
    });
    name_entry
}

fn build(app: &adw::Application, path: &Path) {
    if let Some(w) = app.active_window() {
        w.present();
        return;
    }
    let loaded = ConfigDocument::load(path);

    let banner = adw::Banner::new("");
    let toasts = adw::ToastOverlay::new();
    let page = adw::PreferencesPage::new();

    let trigger_group = adw::PreferencesGroup::builder()
        .title("Trigger")
        .description("The characters that start a search, such as :: or ;;. Use 1 to 3 characters that are not letters, digits, _, + or -.")
        .build();
    let trigger = adw::EntryRow::builder()
        .title("Trigger")
        .show_apply_button(true)
        .build();
    let trigger_error = error_label();
    trigger_group.add(&trigger);
    trigger_group.add(&trigger_error);

    let alias_group = adw::PreferencesGroup::builder()
        .title("Aliases")
        .description("Your own names for emoji or short texts. Type ::name: to insert one.")
        .build();
    let add = gtk::Button::builder()
        .icon_name("list-add-symbolic")
        .tooltip_text("Add an alias")
        .build();
    add.add_css_class("flat");
    alias_group.set_header_suffix(Some(&add));
    let list = gtk::ListBox::new();
    list.set_selection_mode(gtk::SelectionMode::None);
    list.add_css_class("boxed-list");
    alias_group.add(&list);

    page.add(&trigger_group);
    page.add(&alias_group);

    let body = gtk::Box::new(gtk::Orientation::Vertical, 0);
    body.append(&banner);
    body.append(&page);
    page.set_vexpand(true);
    toasts.set_child(Some(&body));
    let view = adw::ToolbarView::new();
    view.add_top_bar(&adw::HeaderBar::new());
    view.set_content(Some(&toasts));

    let ui: Shared = Rc::new(RefCell::new(Ui {
        path: path.to_path_buf(),
        trigger: trigger.clone(),
        trigger_error,
        list,
        rows: Vec::new(),
        toasts,
        banner: banner.clone(),
        saved: None,
    }));

    match &loaded {
        Ok(doc) => {
            trigger.set_text(
                &doc.raw_trigger()
                    .unwrap_or_else(|| DEFAULT_TRIGGER.to_string()),
            );
            for a in doc.raw_aliases() {
                add_row(&ui, &a.name, &a.text);
            }
            if let Ok(c) = doc.config() {
                ui.borrow_mut().saved = Some((c.trigger, doc.raw_aliases()));
            }
            if doc.config().is_err() {
                // Show where each bad value is. Nothing is written until all
                // are fixed.
                save_soon(&ui);
            }
        }
        Err(e) => {
            let what = match e {
                LoadError::Refused(_) => "Fix it in a text editor and open this window again",
                LoadError::Io(_) => "Torpedo uses the defaults until it can read it",
            };
            banner.set_title(&format!(
                "The config file {} has a problem: {e}. {what}.",
                path.display()
            ));
            banner.set_revealed(true);
            page.set_sensitive(false);
        }
    }

    let u = ui.clone();
    trigger.connect_apply(move |_| save_soon(&u));
    let u = ui.clone();
    add.connect_clicked(move |_| {
        let entry = add_row(&u, "", "");
        entry.grab_focus();
    });

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Torpedo Settings")
        .default_width(560)
        .default_height(640)
        .content(&view)
        .build();
    window.present();
}

/// Open the window on the config file at `path`.
pub fn run(path: PathBuf) -> ExitCode {
    let app = adw::Application::builder().application_id(APP_ID).build();
    app.connect_activate(move |app| build(app, &path));
    let none: [&str; 0] = [];
    if app.run_with_args(&none) == glib::ExitCode::SUCCESS {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows(r: &[(&str, &str)]) -> Vec<RowValues> {
        r.iter()
            .map(|(a, b)| RowValues {
                name: a.to_string(),
                text: b.to_string(),
                editing: false,
            })
            .collect()
    }

    #[test]
    fn good_values() {
        let c = check_values(";;", &rows(&[("lgtm", "👍"), ("", "")]));
        assert!(c.is_good());
        assert_eq!(c.aliases.len(), 1);
    }

    #[test]
    fn each_bad_value_is_at_its_field() {
        let c = check_values(
            "ab",
            &rows(&[
                ("lgtm", "👍"),
                ("looks good", "👍"),
                ("LGTM", "✅"),
                ("x", ""),
            ]),
        );
        assert!(c.trigger.is_some());
        assert_eq!(c.rows[0], None);
        assert!(c.rows[1].is_some());
        assert_eq!(c.rows[2].as_deref(), Some("another alias has this name"));
        assert!(c.rows[3].is_some());
        assert!(!c.is_good());
    }

    #[test]
    fn a_row_in_edit_waits_without_a_problem() {
        let mut r = rows(&[("lgtm", "👍"), ("ship", "")]);
        r[1].editing = true;
        let c = check_values("::", &r);
        assert!(c.is_good());
        assert!(c.pending);
        assert!(!c.can_save());
        r[1].editing = false;
        let c = check_values("::", &r);
        assert!(c.rows[1].is_some());
        assert!(!c.can_save());
    }
}
