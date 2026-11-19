#!/usr/bin/env python3
"""The e2e driver: a private ibus-daemon and an IBus client, over JSON lines.

Node starts this script through session.mjs, inside `dbus-run-session` with
e2e/harness/session.conf, and with every IBus and XDG path in a temp folder.
It reads one JSON command per line on stdin and writes one JSON reply per line
on stdout.

Safety: the script refuses to start unless it runs on a private session bus
and every path the daemon writes is under its run folder. It stops the daemon
with SIGTERM and waits for it on "stop", at the end of stdin, and on SIGTERM,
SIGINT or SIGHUP. So the daemon never dies with its bus, which would show the
owner a crash dialog.
"""
import json
import os
import shutil
import signal
import subprocess
import sys
import time

RUN = os.environ.get("TORPEDO_E2E_RUN", "")
ENGINE = os.environ.get("TORPEDO_ENGINE", "")


def refuse(why):
    sys.stderr.write("driver: refusing to run: %s\n" % why)
    sys.exit(2)


if not RUN or not os.path.isdir(RUN):
    refuse("TORPEDO_E2E_RUN is not a folder")
if os.environ.get("DBUS_SESSION_BUS_ADDRESS", "") in ("", os.environ.get("TORPEDO_E2E_OUTER_BUS", "")):
    refuse("not inside a private dbus-run-session")
for var in ("IBUS_ADDRESS_FILE", "XDG_CONFIG_HOME", "XDG_CACHE_HOME", "XDG_DATA_HOME"):
    if not os.environ.get(var, "").startswith(RUN + "/"):
        refuse("%s is not under %s" % (var, RUN))
if "IBUS_ADDRESS" in os.environ:
    refuse("IBUS_ADDRESS is set")
if not os.access(ENGINE, os.X_OK):
    refuse("TORPEDO_ENGINE is not an executable: %r" % ENGINE)

SOCK = os.path.join(RUN, "bus")
if len(SOCK) >= 107:
    refuse("socket path too long: " + SOCK)
ADDR_FILE = os.environ["IBUS_ADDRESS_FILE"]
COMPONENT_DIR = os.path.join(RUN, "component")
for d in (COMPONENT_DIR, os.environ["XDG_CONFIG_HOME"], os.environ["XDG_CACHE_HOME"], os.environ["XDG_DATA_HOME"]):
    os.makedirs(d, exist_ok=True)
shutil.copy("/usr/share/ibus/component/simple.xml", COMPONENT_DIR)
with open(os.path.join(COMPONENT_DIR, "torpedo.xml"), "w") as f:
    f.write("""<?xml version="1.0" encoding="utf-8"?>
<component>
    <name>org.freedesktop.IBus.Torpedo</name>
    <description>Torpedo (e2e)</description>
    <exec>%s --ibus</exec>
    <version>0.0.0</version>
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
            <description>Inline emoji picker</description>
            <rank>0</rank>
        </engine>
    </engines>
</component>
""" % ENGINE)

denv = dict(os.environ)
denv["IBUS_COMPONENT_PATH"] = COMPONENT_DIR
DAEMON_CMD = [
    "/usr/bin/ibus-daemon",
    "--address=unix:path=" + SOCK,
    "--cache=none",
    "--panel=disable",
    "--emoji-extension=disable",
    "--config=disable",
    "--single",
    "--verbose",
]
daemon_log = open(os.path.join(RUN, "daemon.log"), "w")
daemon = subprocess.Popen(DAEMON_CMD, env=denv, stdout=daemon_log, stderr=subprocess.STDOUT)
spawned = []


def stop_daemon():
    if daemon.poll() is None:
        daemon.send_signal(signal.SIGTERM)
        try:
            daemon.wait(10)
        except subprocess.TimeoutExpired:
            daemon.kill()
            daemon.wait()


def cleanup():
    for p in spawned:
        if p.poll() is None:
            p.terminate()
            try:
                p.wait(5)
            except subprocess.TimeoutExpired:
                p.kill()
    stop_daemon()
    daemon_log.close()


def on_signal(signum, _frame):
    cleanup()
    sys.exit(128 + signum)


for s in (signal.SIGTERM, signal.SIGINT, signal.SIGHUP):
    signal.signal(s, on_signal)


def read_address(path, timeout=10.0):
    deadline = time.time() + timeout
    while time.time() < deadline:
        if os.path.exists(path):
            for line in open(path):
                if line.startswith("IBUS_ADDRESS="):
                    return line.strip().split("=", 1)[1]
        time.sleep(0.05)
    return None


ADDRESS = read_address(ADDR_FILE)
if ADDRESS is None:
    cleanup()
    sys.stderr.write("driver: the daemon wrote no address file\n")
    sys.exit(1)

os.environ["IBUS_ADDRESS"] = ADDRESS

import gi  # noqa: E402

gi.require_version("IBus", "1.0")
from gi.repository import GLib, IBus  # noqa: E402

LOOP = GLib.MainContext.default()


def pump(seconds):
    end = time.time() + seconds
    while time.time() < end:
        while LOOP.pending():
            LOOP.iteration(False)
        time.sleep(0.005)


def wait_for(pred, timeout):
    end = time.time() + timeout
    while time.time() < end:
        while LOOP.pending():
            LOOP.iteration(False)
        if pred():
            return True
        time.sleep(0.005)
    return False


IBus.init()
bus = IBus.Bus.new()
if not wait_for(bus.is_connected, 10):
    cleanup()
    sys.stderr.write("driver: cannot connect to the private daemon\n")
    sys.exit(1)

# A panel service, so that a test can click a candidate. Only the owner of
# org.freedesktop.IBus.Panel can send CandidateClicked.
panel = IBus.PanelService(connection=bus.get_connection(), object_path=IBus.PATH_PANEL)
bus.request_name(IBus.SERVICE_PANEL, 0)

events = []
ctx = None
contexts = []
CAPS = (IBus.Capabilite.PREEDIT_TEXT | IBus.Capabilite.LOOKUP_TABLE
        | IBus.Capabilite.FOCUS | IBus.Capabilite.SURROUNDING_TEXT)


def text_event(kind, text, extra=None):
    e = {"type": kind, "text": text.get_text()}
    attrs = []
    lst = text.get_attributes()
    if lst is not None:
        i = 0
        while True:
            a = lst.get(i)
            if a is None:
                break
            attrs.append([a.get_attr_type(), a.get_value(), a.get_start_index(), a.get_end_index()])
            i += 1
    e["attrs"] = attrs
    if extra:
        e.update(extra)
    return e


def table_event(table, visible):
    n = table.get_number_of_candidates()
    labels = []
    for i in range(table.get_page_size()):
        label = table.get_label(i)
        if label is None or label.get_text() == "":
            continue
        labels.append(label.get_text())
    return {
        "type": "table",
        "items": [table.get_candidate(i).get_text() for i in range(n)],
        "cursor": table.get_cursor_pos(),
        "page_size": table.get_page_size(),
        "labels": labels,
        "orientation": table.get_orientation(),
        "visible": bool(visible),
    }


def watch(ic, name):
    def add(e):
        e["context"] = name
        events.append(e)

    ic.connect("commit-text", lambda _ic, t: add(text_event("commit", t)))
    ic.connect("update-preedit-text",
               lambda _ic, t, c, v: add(text_event("preedit", t, {"cursor": c, "visible": bool(v)})))
    ic.connect("show-preedit-text", lambda _ic: add({"type": "show-preedit"}))
    ic.connect("hide-preedit-text", lambda _ic: add({"type": "hide-preedit"}))
    ic.connect("update-lookup-table", lambda _ic, t, v: add(table_event(t, v)))
    ic.connect("show-lookup-table", lambda _ic: add({"type": "show-table"}))
    ic.connect("hide-lookup-table", lambda _ic: add({"type": "hide-table"}))
    ic.connect("forward-key-event",
               lambda _ic, kv, kc, st: add({"type": "forward", "keyval": kv, "keycode": kc, "state": st}))
    ic.connect("require-surrounding-text", lambda _ic: add({"type": "require-surrounding-text"}))


def settle(quiet=0.06, limit=2.0):
    """Pump until no event arrived for `quiet` seconds."""
    end = time.time() + limit
    seen = len(events)
    last = time.time()
    while time.time() < end:
        while LOOP.pending():
            LOOP.iteration(False)
        if len(events) != seen:
            seen = len(events)
            last = time.time()
        elif time.time() - last >= quiet:
            return
        time.sleep(0.005)


def take():
    settle()
    out = list(events)
    events.clear()
    return out


def engine_pids():
    pids = []
    for entry in os.listdir("/proc"):
        if not entry.isdigit():
            continue
        try:
            with open("/proc/%s/stat" % entry) as f:
                stat = f.read()
            ppid = int(stat[stat.rindex(")") + 2:].split()[1])
            with open("/proc/%s/cmdline" % entry, "rb") as f:
                cmd = f.read().split(b"\0")
        except (OSError, ValueError):
            continue
        if ppid == daemon.pid and cmd and cmd[0] == ENGINE.encode():
            pids.append(int(entry))
    return pids


def pid_alive(pid):
    try:
        with open("/proc/%d/stat" % pid) as f:
            return f.read().split(") ")[1].split()[0] != "Z"
    except OSError:
        return False


def do_context(cmd):
    global ctx
    name = cmd.get("name", "ic%d" % (len(contexts) + 1))
    ic = bus.create_input_context(name)
    watch(ic, name)
    ic.set_capabilities(CAPS)
    contexts.append(ic)
    ctx = ic
    ic.focus_in()
    engine = bus.get_global_engine()
    if engine is None or engine.get_name() != "torpedo":
        bus.set_global_engine("torpedo")
    ok = wait_for(lambda: ic.get_engine() is not None and ic.get_engine().get_name() == "torpedo", 10)
    pump(0.2)
    return {"ok": ok, "events": take(), "needs_surrounding_text": ic.needs_surrounding_text()}


def do_key(cmd):
    replies = []

    def done(ic, res, _data):
        try:
            handled = ic.process_key_event_async_finish(res)
        except GLib.Error as e:
            handled = None
            replies.append(str(e))
        events.append({"type": "reply", "handled": handled})
        replies.append(handled)

    ctx.process_key_event_async(cmd["keyval"], cmd.get("keycode", 0), cmd.get("state", 0), -1, None, done, None)
    got = wait_for(lambda: replies, 5)
    return {"ok": got, "events": take()}


def handle(cmd):
    c = cmd["cmd"]
    if c == "context":
        return do_context(cmd)
    if c == "key":
        return do_key(cmd)
    if c == "surrounding":
        text = cmd["text"]
        cursor = cmd.get("cursor", len(text))
        ctx.set_surrounding_text(IBus.Text.new_from_string(text), cursor, cmd.get("anchor", cursor))
        return {"ok": True, "events": take()}
    if c == "focus_in":
        ctx.focus_in()
        return {"ok": True, "events": take()}
    if c == "focus_out":
        ctx.focus_out()
        return {"ok": True, "events": take()}
    if c == "reset":
        ctx.reset()
        return {"ok": True, "events": take()}
    if c == "switch_engine":
        ok = bus.set_global_engine(cmd["name"])
        pump(0.3)
        return {"ok": bool(ok), "events": take()}
    if c == "click":
        panel.candidate_clicked(cmd["index"], cmd.get("button", 1), cmd.get("state", 0))
        return {"ok": True, "events": take()}
    if c == "engine_pids":
        return {"ok": True, "pids": engine_pids()}
    if c == "address":
        return {"ok": True, "address": ADDRESS, "file": ADDR_FILE}
    if c == "spawn_engine":
        env = {k: v for k, v in os.environ.items() if k not in cmd.get("unset", [])}
        env.update(cmd.get("env", {}))
        log = open(os.path.join(RUN, "engine-%d.log" % len(spawned)), "w")
        p = subprocess.Popen([ENGINE] + cmd.get("args", []), env=env, stdout=log, stderr=subprocess.STDOUT)
        spawned.append(p)
        return {"ok": True, "pid": p.pid, "index": len(spawned) - 1}
    if c == "has_owner":
        owned = wait_for(lambda: bus.name_has_owner(cmd["name"]), cmd.get("timeout", 0))
        return {"ok": True, "owned": bool(owned or bus.name_has_owner(cmd["name"]))}
    if c == "wait_spawned":
        p = spawned[cmd["index"]]
        try:
            code = p.wait(cmd.get("timeout", 5))
        except subprocess.TimeoutExpired:
            code = None
        return {"ok": True, "exit": code}
    if c == "stop_spawned":
        p = spawned[cmd["index"]]
        if p.poll() is None:
            p.terminate()
            p.wait(5)
        gone = wait_for(lambda: not bus.name_has_owner(cmd.get("name", "org.freedesktop.IBus.Torpedo")), 5)
        return {"ok": True, "exit": p.returncode, "released": gone}
    if c == "stop_daemon":
        pids = engine_pids()
        stop_daemon()
        end = time.time() + cmd.get("timeout", 5)
        while time.time() < end and any(pid_alive(p) for p in pids):
            time.sleep(0.02)
        return {"ok": True, "engines": pids, "alive": [p for p in pids if pid_alive(p)]}
    raise ValueError("unknown command %r" % c)


def reply(obj):
    try:
        sys.stdout.write(json.dumps(obj) + "\n")
        sys.stdout.flush()
    except BrokenPipeError:
        cleanup()
        os._exit(0)


reply({"ready": True, "address": ADDRESS})
try:
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        cmd = json.loads(line)
        if cmd["cmd"] == "stop":
            break
        try:
            reply(handle(cmd))
        except Exception as e:  # noqa: BLE001
            reply({"ok": False, "error": "%s: %s" % (type(e).__name__, e)})
finally:
    cleanup()
reply({"stopped": True})
