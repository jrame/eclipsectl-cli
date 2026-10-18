# eclipsectl daemon

A normal command starts the same executable as `eclipsectl daemon serve` if needed. No separate daemon executable, Windows service or login hook is installed.

```sh
eclipsectl daemon status
eclipsectl daemon stop
eclipsectl daemon restart
eclipsectl daemon serve --verbose
```

`status` and `stop` do not start a missing daemon. `stop` acknowledges shutdown and drains active calls. `restart` waits up to 15 seconds; if work is still draining it returns an actionable error. Restart an older daemon after upgrading the CLI.

## Lifecycle and isolation

- WSL: Unix socket under `$XDG_RUNTIME_DIR/eclipsectl-cli`, falling back to the user's cache directory; directory mode 0700, socket mode 0600.
- Windows: local named pipe with a current-user SID ACL; remote clients rejected. Identity includes the runtime directory for isolated instances.
- Cache fallback: native cache directory plus `eclipsectl-cli/run`.
- `ECLIPSECTL_RUNTIME_DIR` overrides runtime storage for tests or isolated instances.
- Startup and lifetime file locks prevent competing launches and protect stale-socket cleanup.
- Shutdown after five minutes idle, never during an operation. Unix SIGINT/SIGTERM and foreground Windows Ctrl+C request graceful shutdown.

Each request carries its effective connection configuration. Sessions are keyed by endpoint, authentication identity, MCP server and connection timeout; operation timeout applies per request. Changing projects or credentials never silently retains the first caller's configuration.

Calls sharing a session are serialized to preserve read gates and debug context. Different sessions can progress concurrently. Credentials stay in memory and private IPC; they are not written into metadata or process arguments.

## Protocol and failures

IPC version 1: four-byte big-endian length followed by UTF-8 JSON, limited to 64 MiB per frame. Requests include protocol version and action (`status`, `shutdown`, `doctor`, `call`). Responses share the CLI's `ok/result/error` envelope. Partial reads and payloads above 100 KiB are supported.

Startup does not contact Eclipse; sessions open lazily. Initialization defaults to 30 seconds, operations to 300 seconds. Queueing has a separate operation budget. The client allows queueing, initialization and execution rather than the old five-second IPC limit.

Transport errors and operation timeouts invalidate the session. The next call reconnects. Failed calls are **never replayed**, including after session-expiration HTTP 404 responses: Eclipse might already have executed an edit, build or launch. Re-read before retrying a Java edit after reconnecting.

No persistent credential-bearing logs are written. For diagnostics, stop the daemon and run `eclipsectl daemon serve --verbose`. Application logs use stderr; dependency trace logs are disabled.
