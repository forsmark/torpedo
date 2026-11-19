#!/bin/sh
# Remove what install.sh added.
#
#   ./uninstall.sh            remove the binaries, the desktop file and the
#                             component file
#   ./uninstall.sh --dry-run  print each step and remove nothing
#
# It never restarts IBus and never changes input sources. It takes the same
# PREFIX, COMPONENT_DIR and SUDO overrides as install.sh.
set -eu

DRY_RUN=0
for arg in "$@"; do
  case $arg in
    --dry-run) DRY_RUN=1 ;;
    -h | --help)
      sed -n '2,9s/^# \{0,1\}//p' "$0"
      exit 0
      ;;
    *)
      echo "uninstall.sh: unknown argument: $arg" >&2
      exit 2
      ;;
  esac
done

PREFIX=${PREFIX:-$HOME/.local}
case $PREFIX in /*) ;; *) PREFIX=$(pwd)/$PREFIX ;; esac
COMPONENT_DIR=${COMPONENT_DIR:-/usr/share/ibus/component}
SUDO=${SUDO-sudo}
LIB_DIR=$PREFIX/lib/torpedo
DESKTOP=$PREFIX/share/applications/ibus-setup-torpedo.desktop
COMPONENT=$COMPONENT_DIR/torpedo.xml

run() {
  echo "+ $*"
  if [ "$DRY_RUN" = 0 ]; then "$@"; fi
}

run_sudo() {
  if [ -n "$SUDO" ]; then run $SUDO "$@"; else run "$@"; fi
}

run rm -f "$LIB_DIR/ibus-engine-torpedo" "$LIB_DIR/torpedo-settings"
run rm -f "$DESKTOP"
if [ -d "$LIB_DIR" ] || [ "$DRY_RUN" = 1 ]; then
  run rmdir "$LIB_DIR"
fi
if [ -e "$COMPONENT" ] || [ "$DRY_RUN" = 1 ]; then
  run_sudo rm -f "$COMPONENT"
fi

if [ "$DRY_RUN" = 1 ]; then
  echo
  echo "Dry run: nothing was removed."
fi

cat <<EOT

Torpedo is removed.

Next steps:
  1. Open GNOME Settings, Keyboard, Input Sources, and remove Torpedo from the input sources.
  2. Restart IBus:
       ibus restart
EOT
