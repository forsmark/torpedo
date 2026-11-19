## MODIFIED Requirements

### Requirement: Install writes the component file
`install.sh` SHALL write `torpedo.xml` into `$COMPONENT_DIR` through `$SUDO`. The file SHALL start the installed binary by its absolute path with `--ibus`, and SHALL name one engine `torpedo` with the layout `dk`. The engine's `<setup>` SHALL be the absolute path of the installed settings window. `COMPONENT_DIR` SHALL default to `/usr/share/ibus/component`, and `SUDO` to `sudo`.

#### Scenario: The component file
- **WHEN** the owner runs `install.sh`
- **THEN** `$COMPONENT_DIR/torpedo.xml` exists
- **AND** its `<exec>` is the absolute path of the installed binary followed by `--ibus`
- **AND** it names the engine `torpedo` with `<layout>dk</layout>`

#### Scenario: The component file names the settings window
- **WHEN** the owner runs `install.sh`
- **THEN** the engine in `$COMPONENT_DIR/torpedo.xml` has a `<setup>` with the absolute path `$PREFIX/lib/torpedo/torpedo-settings`

#### Scenario: The component file goes through sudo
- **GIVEN** `SUDO` names a command that records its arguments
- **WHEN** the owner runs `install.sh`
- **THEN** that command wrote the component file

#### Scenario: The default component folder
- **GIVEN** `COMPONENT_DIR` and `SUDO` are not set
- **WHEN** `install.sh` prints its plan with `--dry-run`
- **THEN** the plan copies the component file to `/usr/share/ibus/component/torpedo.xml` with `sudo`
- **AND** the script writes nothing

### Requirement: Install prints the next steps
`install.sh` SHALL end by printing the owner's next steps: restart IBus with `ibus restart`, then add Torpedo as an input source in GNOME Settings and keep Danish as a fallback. It SHALL then say that the settings window opens from the app launcher, from the input source in GNOME Settings and from the input source menu.

#### Scenario: The next steps after install
- **WHEN** the owner runs `install.sh`
- **THEN** the output ends with the steps `ibus restart`, adding Torpedo in GNOME Settings, and keeping Danish as a fallback

#### Scenario: The settings window in the next steps
- **WHEN** the owner runs `install.sh`
- **THEN** the output after the steps names "Torpedo Settings" in the app launcher, GNOME Settings and the input source menu

### Requirement: Uninstall removes what install added
`uninstall.sh` SHALL remove the installed binary, the settings window, their folder and the desktop file, and remove the component file through `$SUDO`. It SHALL print the owner's next steps: remove Torpedo from the input sources and restart IBus.

#### Scenario: Uninstall after install
- **GIVEN** the owner ran `install.sh`
- **WHEN** the owner runs `uninstall.sh`
- **THEN** `$PREFIX/lib/torpedo` and `$COMPONENT_DIR/torpedo.xml` no longer exist

#### Scenario: Uninstall removes the desktop file
- **GIVEN** the owner ran `install.sh`
- **WHEN** the owner runs `uninstall.sh`
- **THEN** `$PREFIX/share/applications/ibus-setup-torpedo.desktop` no longer exists

#### Scenario: The next steps after uninstall
- **WHEN** the owner runs `uninstall.sh`
- **THEN** the output names removing Torpedo from the input sources and `ibus restart`

## ADDED Requirements

### Requirement: Install copies the settings window
`install.sh` SHALL build the release binary of the settings window and copy it to `$PREFIX/lib/torpedo/torpedo-settings`, next to the engine.

#### Scenario: The settings window is installed
- **GIVEN** `PREFIX` names an empty folder
- **WHEN** the owner runs `install.sh`
- **THEN** `$PREFIX/lib/torpedo/torpedo-settings` exists and is executable

### Requirement: Install adds the settings desktop file
`install.sh` SHALL write `$PREFIX/share/applications/ibus-setup-torpedo.desktop` without `$SUDO`. The file SHALL be named "Torpedo Settings" and SHALL start the installed settings window by its absolute path. GNOME Settings shows a settings button for the input source only when a desktop file with this name exists.

#### Scenario: The desktop file
- **WHEN** the owner runs `install.sh`
- **THEN** `$PREFIX/share/applications/ibus-setup-torpedo.desktop` exists
- **AND** its `Name` is `Torpedo Settings` and its `Exec` is the absolute path of the installed settings window

#### Scenario: The desktop file goes without sudo
- **GIVEN** `SUDO` names a command that records its arguments
- **WHEN** the owner runs `install.sh`
- **THEN** that command did not write the desktop file
