# S007 — Co-located background worker tasks fail silently

| Field                    | Value                                                                                                                                                                                                                                                            |
|--------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Priority                 | medium                                                                                                                                                                                                                                                           |
| File                     | `apps/retailer-sourcing/docs/implementation-plans/002-daily-sourcing-flow.md`                                                                                                                                                                                    |
| Decision                 | accepted                                                                                                                                                                                                                                                         |
| Implementation reference | `apps/retailer-sourcing/docs/implementation-plans/002-daily-sourcing-flow.md` (Worker loops section now retains `JoinHandle`s and includes them in the main `tokio::select!`; any worker or HTTP server exit triggers graceful shutdown via `handle.shutdown()`) |
| Created at               | 2026-05-25                                                                                                                                                                                                                                                       |
| Author                   | Gemini, gemini-3.5-flash, medium                                                                                                                                                                                                                                 |
| Reviewer                 |                                                                                                                                                                                                                                                                  |

## Issue

In the "Worker loops in the binary" section, the plan spawns the command and event workers using `tokio::spawn` and discards their `JoinHandle`s:

```rust
let token = handle.child_token();
tokio::spawn(kernel::io::run_command_worker(
    handle.command_consumer(),
    token.clone(),
));
tokio::spawn(kernel::io::run_event_worker(
    handle.event_consumer(),
    token.clone(),
));
```

If either worker exits due to a fatal error (e.g. database connectivity loss, persistent panics, or configuration issue), the worker loop will terminate silently. Meanwhile, the HTTP server will continue running and responding `200 OK` or `202 Accepted` to health checks and incoming requests, but no commands or events will ever be processed. This leads to extremely hard-to-detect partial outages in production.

## Suggestion

Monitor the spawned background tasks by including them in the main `tokio::select!` block, or by managing their lifecycles. If any of the background workers or the web server exits, trigger a graceful shutdown of the other components and exit the process with a non-zero code.

For example, update the startup pattern to track task handles:

```rust
let token = handle.child_token();
let command_worker = tokio::spawn(kernel::io::run_command_worker(
    handle.command_consumer(),
    token.clone(),
));
let event_worker = tokio::spawn(kernel::io::run_event_worker(
    handle.event_consumer(),
    token.clone(),
));

let state = Arc::new(handle.state());
let bearer = Arc::new(bearer_token);

tokio::select! {
    res = Server::new(TcpListener::bind(BIND_ADDRESS)).run(app(state, bearer)) => {
        if let Err(e) = res {
            eprintln!("HTTP server error: {e}");
        }
    }
    res = command_worker => {
        eprintln!("Command worker exited unexpectedly: {:?}", res);
    }
    res = event_worker => {
        eprintln!("Event worker exited unexpectedly: {:?}", res);
    }
    _ = tokio::signal::ctrl_c() => {
        println!("Received SIGINT, shutting down...");
    }
}

handle.shutdown();
handle.wait().await?;
```
