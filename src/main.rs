//! The application sidecar (docs/protocol.md): one application per surface, which speaks the embed protocol on a Unix
//! socket that its environment names, relayed to the host on standard input and output.

use std::collections::HashMap;
use std::os::unix::process::ExitStatusExt;
use std::path::PathBuf;
use std::process::{ExitStatus, Stdio};
use std::time::Duration;

use serde_json::{Value, json};
use soksak_embed::client::SOCKET_VARIABLE;
use soksak_embed::protocol::{
    FromApplication, FromSidecar, decode_from_application, decode_from_sidecar, encode,
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines};
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{UnixListener, UnixStream};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;

/// The time an application has to end after `end` before the sidecar ends it.
const STOP: Duration = Duration::from_secs(2);

/// A line to standard output.
type Out = mpsc::UnboundedSender<String>;

/// A request that the main loop passes to the task of a surface.
enum Control {
    Configure {
        name: String,
        generation: u64,
        raster: u64,
        width: u32,
        height: u32,
        scale: f64,
    },
    Answer {
        sequence: u64,
        error: Option<String>,
    },
    Input(FromSidecar),
    /// Ends the application; `answer` is true for a `closed` of the host, which the sidecar answers, and false when
    /// standard input ended.
    Close {
        answer: bool,
    },
}

struct Surface {
    command: Vec<String>,
    control: mpsc::UnboundedSender<Control>,
}

fn body(out: &Out, surface: &str, body: Value) {
    let _ = out.send(json!({"surface": surface, "body": body}).to_string());
}

fn error(out: &Out, surface: &str, text: impl Into<String>) {
    body(out, surface, json!({"error": text.into()}));
}

fn exit_text(status: ExitStatus) -> String {
    match (status.code(), status.signal()) {
        (Some(code), _) => format!("exit status {code}"),
        (None, Some(signal)) => format!("signal {signal}"),
        (None, None) => "exit status unknown".to_string(),
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let (out, mut lines_out) = mpsc::unbounded_channel::<String>();
    let writer = tokio::spawn(async move {
        let mut stdout = tokio::io::stdout();
        while let Some(line) = lines_out.recv().await {
            if stdout
                .write_all(format!("{line}\n").as_bytes())
                .await
                .is_err()
                || stdout.flush().await.is_err()
            {
                eprintln!("error: soksak-app: standard output closed");
                return;
            }
        }
    });
    let mut surfaces: HashMap<String, Surface> = HashMap::new();
    // The configure of a surface whose application is not running: the host configures the region when the page creates
    // its composition, which can be before the open of the page, and the next open passes it on.
    let mut configured: HashMap<String, Control> = HashMap::new();
    let mut tasks = tokio::task::JoinSet::new();
    let mut input = BufReader::new(tokio::io::stdin()).lines();
    let mut count = 0u64;
    loop {
        let line = match input.next_line().await {
            Ok(Some(line)) => line,
            Ok(None) => break,
            Err(failure) => {
                eprintln!("error: soksak-app: standard input: {failure}");
                break;
            }
        };
        let message: Value = match serde_json::from_str(&line) {
            Ok(message) => message,
            Err(failure) => {
                eprintln!("error: soksak-app: request {line}: {failure}");
                continue;
            }
        };
        let Some(surface) = message["surface"].as_str().map(str::to_string) else {
            eprintln!("error: soksak-app: request without surface: {line}");
            continue;
        };
        if message["closed"] == true {
            configured.remove(&surface);
            match surfaces.remove(&surface) {
                Some(entry) if entry.control.send(Control::Close { answer: true }).is_ok() => {}
                _ => {
                    let _ = out.send(json!({"surface": surface, "closed": true}).to_string());
                }
            }
            continue;
        }
        let request = &message["body"];
        if let Some(image) = request.get("image") {
            let control = if let Some(c) = image.get("configure") {
                match (
                    c["name"].as_str(),
                    c["generation"].as_u64(),
                    c["raster"].as_u64(),
                    c["width"].as_u64(),
                    c["height"].as_u64(),
                    c["scale"].as_f64(),
                ) {
                    (
                        Some(name),
                        Some(generation),
                        Some(raster),
                        Some(width),
                        Some(height),
                        Some(scale),
                    ) => Control::Configure {
                        name: name.to_string(),
                        generation,
                        raster,
                        width: width as u32,
                        height: height as u32,
                        scale,
                    },
                    _ => {
                        error(&out, &surface, format!("configure: invalid {c}"));
                        continue;
                    }
                }
            } else if let Some(consumed) = image.get("consumed") {
                Control::Answer {
                    sequence: consumed["sequence"].as_u64().unwrap_or(0),
                    error: None,
                }
            } else if let Some(kind) = image["error"].as_str() {
                Control::Answer {
                    sequence: image["sequence"].as_u64().unwrap_or(0),
                    error: Some(kind.to_string()),
                }
            } else {
                error(&out, &surface, format!("unknown image answer {image}"));
                continue;
            };
            match surfaces.get(&surface) {
                Some(entry) if !entry.control.is_closed() => {
                    let _ = entry.control.send(control);
                }
                _ if matches!(control, Control::Configure { .. }) => {
                    configured.insert(surface, control);
                }
                _ => error(
                    &out,
                    &surface,
                    format!("image: the surface {surface} has no running application"),
                ),
            }
            continue;
        }
        match request["operation"].as_str() {
            Some("open") => {
                let command: Vec<String> = request["command"]
                    .as_array()
                    .map(|items| {
                        items
                            .iter()
                            .filter_map(|item| item.as_str().map(str::to_string))
                            .collect()
                    })
                    .unwrap_or_default();
                if command.is_empty() {
                    error(&out, &surface, "open: command must name a program");
                    continue;
                }
                if let Some(entry) = surfaces.get(&surface)
                    && !entry.control.is_closed()
                {
                    if entry.command != command {
                        error(
                            &out,
                            &surface,
                            format!(
                                "open: the surface {surface} runs {:?}, not {command:?}",
                                entry.command
                            ),
                        );
                    }
                    continue;
                }
                count += 1;
                let socket = std::env::temp_dir()
                    .join(format!("soksak-app-{}-{count}.sock", std::process::id()));
                let (control, receiver) = mpsc::unbounded_channel();
                if let Some(configure) = configured.remove(&surface) {
                    let _ = control.send(configure);
                }
                tasks.spawn(run(
                    surface.clone(),
                    command.clone(),
                    socket,
                    receiver,
                    out.clone(),
                ));
                surfaces.insert(surface, Surface { command, control });
            }
            Some("input") => {
                let line = request["message"].to_string();
                let message = match decode_from_sidecar(&line) {
                    Ok(message) => message,
                    Err(failure) => {
                        error(&out, &surface, format!("input: {failure}"));
                        continue;
                    }
                };
                if !matches!(
                    message,
                    FromSidecar::Key { .. }
                        | FromSidecar::Compose { .. }
                        | FromSidecar::Commit { .. }
                        | FromSidecar::Pointer { .. }
                        | FromSidecar::Scroll { .. }
                        | FromSidecar::Focus { .. }
                ) {
                    let kind = request["message"]["type"]
                        .as_str()
                        .unwrap_or("")
                        .to_string();
                    error(
                        &out,
                        &surface,
                        format!("input: {kind} is not an input message"),
                    );
                    continue;
                }
                match surfaces.get(&surface) {
                    Some(entry) if entry.control.send(Control::Input(message)).is_ok() => {}
                    _ => error(
                        &out,
                        &surface,
                        format!("input: the surface {surface} has no running application"),
                    ),
                }
            }
            _ => error(&out, &surface, format!("unknown request {request}")),
        }
    }
    for (_, entry) in surfaces.drain() {
        let _ = entry.control.send(Control::Close { answer: false });
    }
    while tasks.join_next().await.is_some() {}
    drop(out);
    let _ = writer.await;
}

/// The connection of an application after it connected.
struct Connection {
    lines: Lines<BufReader<OwnedReadHalf>>,
    writer: OwnedWriteHalf,
    hello: bool,
}

/// The region that the host configured last.
struct Region {
    name: String,
    generation: u64,
    raster: u64,
    width: u32,
    height: u32,
    scale: f64,
}

async fn send(connection: &mut Option<Connection>, message: &FromSidecar) -> Result<(), String> {
    let Some(connection) = connection else {
        return Err("the application has not connected".into());
    };
    let line = encode(message).map_err(|failure| failure.to_string())?;
    connection
        .writer
        .write_all(line.as_bytes())
        .await
        .map_err(|failure| format!("embed socket write: {failure}"))
}

async fn next_line(connection: &mut Option<Connection>) -> std::io::Result<Option<String>> {
    match connection {
        Some(connection) => connection.lines.next_line().await,
        None => std::future::pending().await,
    }
}

/// Runs the application of one surface until it ends.
async fn run(
    surface: String,
    command: Vec<String>,
    socket: PathBuf,
    mut control: mpsc::UnboundedReceiver<Control>,
    out: Out,
) {
    let _ = std::fs::remove_file(&socket);
    let listener = match UnixListener::bind(&socket) {
        Ok(listener) => listener,
        Err(failure) => {
            return error(
                &out,
                &surface,
                format!("embed socket {}: {failure}", socket.display()),
            );
        }
    };
    // The application gets no pipe of the host and runs in its own process group (docs/spec of sidecars).
    let spawned = Command::new(&command[0])
        .args(&command[1..])
        .env(SOCKET_VARIABLE, &socket)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .process_group(0)
        .kill_on_drop(true)
        .spawn();
    let mut child: Child = match spawned {
        Ok(child) => child,
        Err(failure) => {
            let _ = std::fs::remove_file(&socket);
            return error(&out, &surface, format!("open: {}: {failure}", command[0]));
        }
    };
    body(
        &out,
        &surface,
        json!({"event": "started", "pid": child.id()}),
    );
    let mut connection: Option<Connection> = None;
    let mut region: Option<Region> = None;
    // The input of the last frame that was recorded on standard error.
    let mut recorded: Option<u64> = None;
    let mut stopping = false;
    let mut closing = false;
    let mut answer = false;
    let stop = tokio::time::sleep(Duration::MAX / 4);
    tokio::pin!(stop);
    loop {
        tokio::select! {
            accepted = listener.accept(), if connection.is_none() => match accepted {
                Ok((stream, _)) => {
                    let (read, writer) = UnixStream::into_split(stream);
                    connection = Some(Connection { lines: BufReader::new(read).lines(), writer, hello: false });
                }
                Err(failure) => error(&out, &surface, format!("embed socket accept: {failure}")),
            },
            line = next_line(&mut connection) => match line {
                Ok(Some(line)) => {
                    if let Err(failure) = application_line(&surface, &out, &line, &mut connection, &region, &mut recorded).await {
                        error(&out, &surface, format!("application: {failure}"));
                        stopping = true;
                        let _ = child.start_kill();
                    }
                }
                Ok(None) => connection = None,
                Err(failure) => {
                    error(&out, &surface, format!("embed socket read: {failure}"));
                    connection = None;
                }
            },
            status = child.wait() => {
                let exit = match status {
                    Ok(status) => exit_text(status),
                    Err(failure) => format!("wait failed: {failure}"),
                };
                // The lines that the application wrote before it ended are relayed first.
                while connection.is_some()
                    && let Ok(Some(line)) = next_line(&mut connection).await
                {
                    if application_line(&surface, &out, &line, &mut connection, &region, &mut recorded).await.is_err() {
                        break;
                    }
                }
                body(&out, &surface, json!({"event": "exited", "exit": exit}));
                if !stopping && !closing {
                    error(&out, &surface, format!("the application {} ended by itself: {exit}", command[0]));
                }
                break;
            }
            _ = &mut stop, if closing => {
                let _ = child.start_kill();
            }
            request = control.recv(), if !closing => match request {
                Some(Control::Configure { name, generation, raster, width, height, scale }) => {
                    region = Some(Region { name, generation, raster, width, height, scale });
                    if connection.as_ref().is_some_and(|connection| connection.hello)
                        && let Err(failure) = send(&mut connection, &FromSidecar::Configure { width, height, scale }).await
                    {
                        error(&out, &surface, failure);
                    }
                }
                Some(Control::Answer { sequence, error: refused }) => {
                    if let Some(kind) = refused.filter(|kind| kind != "stale") {
                        let name = region.as_ref().map_or("", |region| region.name.as_str());
                        error(&out, &surface, format!("the host refused frame {sequence} of the region {name}: {kind}"));
                    }
                    if let Err(failure) = send(&mut connection, &FromSidecar::Consumed { sequence }).await {
                        error(&out, &surface, failure);
                    }
                }
                Some(Control::Input(message)) => {
                    if !connection.as_ref().is_some_and(|connection| connection.hello) {
                        error(&out, &surface, format!("input: the application of {surface} has not connected"));
                    } else if let Err(failure) = send(&mut connection, &message).await {
                        error(&out, &surface, failure);
                    }
                }
                None => {
                    closing = true;
                    if send(&mut connection, &FromSidecar::End).await.is_err() {
                        let _ = child.start_kill();
                    }
                    stop.as_mut().reset(tokio::time::Instant::now() + STOP);
                }
                Some(Control::Close { answer: answering }) => {
                    closing = true;
                    answer = answering;
                    if send(&mut connection, &FromSidecar::End).await.is_err() {
                        let _ = child.start_kill();
                    }
                    stop.as_mut().reset(tokio::time::Instant::now() + STOP);
                }
            },
        }
    }
    let _ = std::fs::remove_file(&socket);
    if answer {
        let _ = out.send(json!({"surface": surface, "closed": true}).to_string());
    }
    // The task ends here and drops its receiver, so the main loop answers each later request of the surface itself; the
    // current-thread runtime runs no other task between the event above and this end.
}

/// Handles one line of the application; an error ends the application.
async fn application_line(
    surface: &str,
    out: &Out,
    line: &str,
    connection: &mut Option<Connection>,
    region: &Option<Region>,
    recorded: &mut Option<u64>,
) -> Result<(), String> {
    let message = decode_from_application(line).map_err(|failure| failure.to_string())?;
    let said_hello = connection
        .as_ref()
        .is_some_and(|connection| connection.hello);
    match message {
        FromApplication::Hello { toolkit, .. } => {
            if said_hello {
                return Err("a second hello".into());
            }
            if let Some(connection) = connection.as_mut() {
                connection.hello = true;
            }
            body(
                out,
                surface,
                json!({"event": "connected", "toolkit": toolkit}),
            );
            if let Some(region) = region {
                let configure = FromSidecar::Configure {
                    width: region.width,
                    height: region.height,
                    scale: region.scale,
                };
                send(connection, &configure).await?;
            }
        }
        _ if !said_hello => {
            return Err(format!(
                "the first message is not hello: {}",
                line.trim_end()
            ));
        }
        FromApplication::Frame {
            sequence,
            token,
            width,
            height,
            scale,
            format,
            input,
        } => {
            // The first frame that shows a new input is recorded, which joins the input of the page to the answer of the
            // host for that sequence; the record grows with the input and not with the frames.
            if input.is_some() && input != *recorded {
                *recorded = input;
                eprintln!("frame {sequence} input {}", input.unwrap_or_default());
            }
            let fits = region.as_ref().filter(|region| {
                (region.width, region.height, region.scale) == (width, height, scale)
            });
            match (region, fits) {
                (_, Some(region)) => body(
                    out,
                    surface,
                    json!({"image": {"name": region.name, "generation": region.generation, "raster": region.raster, "token": token, "width": width, "height": height, "scale": scale, "format": format, "sequence": sequence}}),
                ),
                (Some(region), None) => {
                    error(
                        out,
                        surface,
                        format!(
                            "frame {sequence} is {width} by {height} at scale {scale}, but the region {} is configured as {} by {} at scale {}",
                            region.name, region.width, region.height, region.scale
                        ),
                    );
                    send(connection, &FromSidecar::Consumed { sequence }).await?;
                }
                (None, None) => {
                    error(
                        out,
                        surface,
                        format!("frame {sequence} arrived before the region was configured"),
                    );
                    send(connection, &FromSidecar::Consumed { sequence }).await?;
                }
            }
        }
        FromApplication::Cursor { shape } => {
            body(out, surface, json!({"event": "cursor", "shape": shape}))
        }
        FromApplication::Caret {
            x,
            y,
            width,
            height,
        } => body(
            out,
            surface,
            json!({"event": "caret", "x": x, "y": y, "width": width, "height": height}),
        ),
        FromApplication::State { title } => {
            body(out, surface, json!({"event": "state", "title": title}))
        }
        FromApplication::Error { message } => {
            error(out, surface, format!("application: {message}"))
        }
    }
    Ok(())
}
