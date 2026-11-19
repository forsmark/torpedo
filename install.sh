#!/bin/sh
# Install torpedo for the current user.
#
#   ./install.sh            build, install, and print the next steps
#   ./install.sh --dry-run  print each step and write nothing
#
# It builds the release binary, copies it to $PREFIX/lib/torpedo/, and writes
# the IBus component file to $COMPONENT_DIR with $SUDO. It never restarts IBus
# and never changes input sources: the owner does both by hand afterwards.
#
# Overrides, for tests and other layouts:
#   PREFIX         default ~/.local
#   COMPONENT_DIR  default /usr/share/ibus/component
#   SUDO           default sudo. An empty value runs without sudo.
set -eu

DRY_RUN=0
for arg in "$@"; do
  case $arg in
    --dry-run) DRY_RUN=1 ;;
    -h | --help)
      sed -n '2,15s/^# \{0,1\}//p' "$0"
      exit 0
      ;;
    *)
      echo "install.sh: unknown argument: $arg" >&2
      exit 2
      ;;
  esac
done

ROOT=$(cd "$(dirname "$0")" && pwd)
PREFIX=${PREFIX:-$HOME/.local}
case $PREFIX in /*) ;; *) PREFIX=$(pwd)/$PREFIX ;; esac
COMPONENT_DIR=${COMPONENT_DIR:-/usr/share/ibus/component}
SUDO=${SUDO-sudo}
TARGET=${CARGO_TARGET_DIR:-$ROOT/target}
BINARY=$PREFIX/lib/torpedo/ibus-engine-torpedo
COMPONENT=$COMPONENT_DIR/torpedo.xml

# Print a step, and run it unless this is a dry run.
run() {
  echo "+ $*"
  if [ "$DRY_RUN" = 0 ]; then "$@"; fi
}

# The same, through $SUDO when it is set.
run_sudo() {
  if [ -n "$SUDO" ]; then run $SUDO "$@"; else run "$@"; fi
}

xml_escape() {
  printf '%s' "$1" | sed -e 's/&/\&amp;/g' -e 's/</\&lt;/g' -e 's/>/\&gt;/g'
}

component() {
  cat <<XML
<?xml version="1.0" encoding="utf-8"?>
<component>
    <name>org.freedesktop.IBus.Torpedo</name>
    <description>Torpedo inline emoji picker</description>
    <exec>$(xml_escape "$BINARY") --ibus</exec>
    <version>0.1.0</version>
    <author>torpedo</author>
    <license>MIT</license>
    <homepage>https://github.com/forsmark/torpedo</homepage>
    <textdomain>torpedo</textdomain>
    <engines>
        <engine>
            <name>torpedo</name>
            <language>da</language>
            <license>MIT</license>
            <author>torpedo</author>
            <layout>dk</layout>
            <longname>Torpedo</longname>
            <description>Inline emoji picker. Type :: and a name.</description>
            <icon>face-smile</icon>
            <rank>0</rank>
        </engine>
    </engines>
</component>
XML
}

if ! command -v cargo >/dev/null 2>&1 && [ -f "$HOME/.cargo/env" ]; then
  . "$HOME/.cargo/env"
fi

run cargo build --release -p torpedo-engine --manifest-path "$ROOT/Cargo.toml"
run install -D -m 755 "$TARGET/release/torpedo-engine" "$BINARY"

if [ "$DRY_RUN" = 1 ]; then
  echo "+ $SUDO install -D -m 644 <component file> $COMPONENT"
  component
else
  TMP=$(mktemp)
  trap 'rm -f "$TMP"' EXIT
  component >"$TMP"
  run_sudo install -D -m 644 "$TMP" "$COMPONENT"
fi

if [ "$DRY_RUN" = 1 ]; then
  echo
  echo "Dry run: nothing was written."
fi

cat <<EOT

Torpedo is installed: $BINARY
IBus reads $COMPONENT only when it starts.

Next steps:
  1. Restart IBus:
       ibus restart
  2. Open GNOME Settings, Keyboard, Input Sources, and add Torpedo.
     Keep Danish in the list as a fallback.
EOT
