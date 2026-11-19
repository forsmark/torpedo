# Spec Delta

## Purpose

The daemon connection says how the engine finds the IBus daemon, when it connects to it, and when it stops, so that IBus can start and stop the engine like any other engine.

## ADDED Requirements

### Requirement: The engine connects only when IBus starts it
The engine SHALL connect to an IBus daemon only when it is started with `--ibus`. Without that flag, it SHALL exit without connecting.

#### Scenario: Started with --ibus
- **GIVEN** a running IBus daemon and its address in `IBUS_ADDRESS`
- **WHEN** the engine starts with `--ibus`
- **THEN** the engine registers on that daemon

#### Scenario: Started without --ibus
- **GIVEN** a running IBus daemon and its address in `IBUS_ADDRESS`
- **WHEN** the engine starts without arguments
- **THEN** the engine exits
- **AND** it never registers on the daemon

### Requirement: The engine finds the daemon
The engine SHALL take the daemon address from `IBUS_ADDRESS`. Without it, the engine SHALL read the address file that `IBUS_ADDRESS_FILE` names. Without either, it SHALL read the default address file, as IBus clients do.

#### Scenario: The address in IBUS_ADDRESS
- **GIVEN** `IBUS_ADDRESS` holds the daemon's address and `IBUS_ADDRESS_FILE` names a file with another address
- **WHEN** the engine starts with `--ibus`
- **THEN** it registers on the daemon of `IBUS_ADDRESS`

#### Scenario: The address file in IBUS_ADDRESS_FILE
- **GIVEN** `IBUS_ADDRESS` is not set and `IBUS_ADDRESS_FILE` names the daemon's address file
- **WHEN** the engine starts with `--ibus`
- **THEN** it registers on the daemon

#### Scenario: The default address file
- **GIVEN** neither `IBUS_ADDRESS` nor `IBUS_ADDRESS_FILE` is set, and the daemon's address is in the default address file under `XDG_CONFIG_HOME`
- **WHEN** the engine starts with `--ibus`
- **THEN** it registers on the daemon

### Requirement: The engine stops with the daemon
The engine SHALL exit when its connection to the daemon closes.

#### Scenario: The daemon stops
- **GIVEN** the daemon started the engine
- **WHEN** the daemon stops
- **THEN** the engine process exits
