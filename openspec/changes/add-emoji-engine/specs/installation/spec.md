# Spec Delta

## Purpose

Installation puts the engine and its component file where IBus finds them with one command, and takes them away again, without touching the running IBus session or the input sources.

## ADDED Requirements

### Requirement: Install copies the engine binary
`install.sh` SHALL build the release binary and copy it to `$PREFIX/lib/torpedo/ibus-engine-torpedo`. `PREFIX` SHALL default to `~/.local`.

#### Scenario: The binary is installed
- **GIVEN** `PREFIX` names an empty folder
- **WHEN** the owner runs `install.sh`
- **THEN** `$PREFIX/lib/torpedo/ibus-engine-torpedo` exists and is executable

#### Scenario: The default prefix
- **GIVEN** `PREFIX` is not set and `HOME` names an empty folder
- **WHEN** the owner runs `install.sh`
- **THEN** the binary is at `$HOME/.local/lib/torpedo/ibus-engine-torpedo`

### Requirement: Install writes the component file
`install.sh` SHALL write `torpedo.xml` into `$COMPONENT_DIR` through `$SUDO`. The file SHALL start the installed binary by its absolute path with `--ibus`, and SHALL name one engine `torpedo` with the layout `dk`. `COMPONENT_DIR` SHALL default to `/usr/share/ibus/component`, and `SUDO` to `sudo`.

#### Scenario: The component file
- **WHEN** the owner runs `install.sh`
- **THEN** `$COMPONENT_DIR/torpedo.xml` exists
- **AND** its `<exec>` is the absolute path of the installed binary followed by `--ibus`
- **AND** it names the engine `torpedo` with `<layout>dk</layout>`

#### Scenario: The component file goes through sudo
- **GIVEN** `SUDO` names a command that records its arguments
- **WHEN** the owner runs `install.sh`
- **THEN** that command wrote the component file

#### Scenario: The default component folder
- **GIVEN** `COMPONENT_DIR` and `SUDO` are not set
- **WHEN** `install.sh` prints its plan with `--dry-run`
- **THEN** the plan copies the component file to `/usr/share/ibus/component/torpedo.xml` with `sudo`
- **AND** the script writes nothing

### Requirement: The scripts never restart IBus
`install.sh` and `uninstall.sh` SHALL NOT start, stop or restart IBus, and SHALL NOT run any `ibus` command.

#### Scenario: Install runs no ibus command
- **GIVEN** an `ibus` command on the PATH that records each call
- **WHEN** the owner runs `install.sh`
- **THEN** the `ibus` command was never called

#### Scenario: Uninstall runs no ibus command
- **GIVEN** an `ibus` command on the PATH that records each call
- **WHEN** the owner runs `uninstall.sh`
- **THEN** the `ibus` command was never called

### Requirement: The scripts never change input sources
`install.sh` and `uninstall.sh` SHALL NOT change the input sources or any other setting.

#### Scenario: Install changes no setting
- **GIVEN** `gsettings` and `dconf` commands on the PATH that record each call
- **WHEN** the owner runs `install.sh`
- **THEN** neither command was called

#### Scenario: Uninstall changes no setting
- **GIVEN** `gsettings` and `dconf` commands on the PATH that record each call
- **WHEN** the owner runs `uninstall.sh`
- **THEN** neither command was called

### Requirement: Install prints the next steps
`install.sh` SHALL end by printing the owner's next steps: restart IBus with `ibus restart`, then add Torpedo as an input source in GNOME Settings and keep Danish as a fallback.

#### Scenario: The next steps after install
- **WHEN** the owner runs `install.sh`
- **THEN** the output ends with the steps `ibus restart`, adding Torpedo in GNOME Settings, and keeping Danish as a fallback

### Requirement: Uninstall removes what install added
`uninstall.sh` SHALL remove the installed binary and its folder, and remove the component file through `$SUDO`. It SHALL print the owner's next steps: remove Torpedo from the input sources and restart IBus.

#### Scenario: Uninstall after install
- **GIVEN** the owner ran `install.sh`
- **WHEN** the owner runs `uninstall.sh`
- **THEN** `$PREFIX/lib/torpedo` and `$COMPONENT_DIR/torpedo.xml` no longer exist

#### Scenario: The next steps after uninstall
- **WHEN** the owner runs `uninstall.sh`
- **THEN** the output names removing Torpedo from the input sources and `ibus restart`
