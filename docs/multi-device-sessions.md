# Multi-device sessions

The TUI is one process with one paint surface. Several RouterOS devices are
modeled as isolated sessions under a thin shell, not as shared client or UI
state.

```mermaid
flowchart TB
  subgraph shell [App shell]
    ids[Tab ids plus active]
    shared[Theme profiles credential store file log]
  end
  subgraph rt [Runtime]
    dispatch[Dispatch by SessionId]
  end
  subgraph s1 [Session]
    c1[Own Client]
    u1[Own UI and generations]
    io1[Own watches and StreamGates]
  end
  subgraph s2 [Session]
    c2[Own Client]
    u2[Own UI and generations]
    io2[Own watches and StreamGates]
  end
  shell --> s1
  shell --> s2
  dispatch -->|"cmd and msg carry SessionId"| s1
  dispatch -->|"cmd and msg carry SessionId"| s2
  shared -.->|read write by profile name only| s1
  shared -.->|read write by profile name only| s2
```

## Shell, session, runtime

- **Shell** owns the tab list, the active tab id, and process-wide stores
  (theme, saved profiles, credential store, file log). It does not own a
  RouterOS TCP session.
- **Session** owns one `Client`, one UI tree, request generations, watches,
  and `StreamGate`s. A new tab is `Session::default()` on Login.
- **Runtime** is the I/O loop. Every command and `WorkerMsg` carries a
  `SessionId`. Dispatch applies the message only to that session.

Chrome (`tab_bar` in `mtui-ui`) paints titles and connection flags. It does
not open sockets or interpret tab ids.

## Isolation

- **Ownership.** Do not move widgets, tables, or generations between
  sessions. Closing a tab drops that session's client, watches, and gates.
- **Addressing.** Key, tick, and worker paths stamp `SessionId`. A result
  for tab A never mutates tab B.
- **Secrets.** Credentials stay at the connection boundary. Render models
  and logs do not hold passwords or TOTP.
- **I/O lifetime.** Watches and gates live with the session that started
  them. Navigation inside a session still uses that session's generation.
- **Shared stores.** Theme, profiles, credentials, and the file log are
  keyed by profile name. Sessions read and write those stores; they do not
  share in-memory maps of each other's UI.
- **New tab.** Open at Login via `Session::default()`. Cap open tabs at
  about 8 so chrome and worker load stay bounded.

## Client handles

`Client.clone()` is a cheap handle to the same TCP connection. Never wrap
the client in `Arc<Client>` and hand that out across sessions. Each session
constructs or receives its own client. Cloning inside one session (command
vs watch) is fine; cloning across tab ids is not.

## Paint vs apply

Background tabs still apply their own `WorkerMsg` so tables and generations
stay current. Only the active session is painted. Switching tabs does not
replay I/O; it selects which session `render` reads.
