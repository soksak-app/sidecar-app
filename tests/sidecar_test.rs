//! The application sidecar against a host of this test (docs/protocol.md): it starts an application, relays configure,
//! frames, answers and input, passes the messages of the application to the page, and stops the application on close.
//! The application is this test executable in the role that `SOKSAK_APP_TEST_ROLE` names; it reports each message it
//! receives as a `state` whose title is the line of that message.

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

use serde_json::{Value, json};
use soksak_embed::client::Client;
use soksak_embed::protocol::{FromApplication, FromSidecar, Token, encode};

const ROLE: &str = "SOKSAK_APP_TEST_ROLE";
const STEP: Duration = Duration::from_secs(10);

/// The application role of this executable; it does nothing when the role is not set.
#[test]
fn fake_application() {
    let Ok(role) = std::env::var(ROLE) else {
        return;
    };
    let (mut client, configure) = match Client::from_env("fake") {
        Ok(connected) => connected,
        // A surface closed before its configure ends the application while it waits for configure.
        Err(failure) if failure.ends_with("not configure: End") => return,
        Err(failure) => panic!("{failure}"),
    };
    let report = |client: &mut Client, message: &FromSidecar| {
        let title = encode(message).unwrap().trim_end().to_string();
        client.send(&FromApplication::State { title }).unwrap();
    };
    report(&mut client, &configure);
    let FromSidecar::Configure {
        width,
        height,
        scale,
    } = configure
    else {
        unreachable!()
    };
    let wide = if role == "mismatch" { width + 1 } else { width };
    client
        .send(&FromApplication::Frame {
            sequence: 1,
            token: Token::IosurfaceGlobal {
                id: 1,
                nonce: "AAAAAAAAAAAAAAAAAAAAAA==".into(),
            },
            width: wide,
            height,
            scale,
            format: "bgra8".into(),
            input: None,
        })
        .unwrap();
    client
        .send(&FromApplication::Cursor {
            shape: soksak_embed::protocol::Cursor::Text,
        })
        .unwrap();
    loop {
        let message = client.recv().unwrap();
        report(&mut client, &message);
        if let FromSidecar::Pointer { input, .. } = message {
            // A frame that shows the pointer input, which the sidecar records once.
            for sequence in [2, 3] {
                client
                    .send(&FromApplication::Frame {
                        sequence,
                        token: Token::IosurfaceGlobal {
                            id: 1,
                            nonce: "AAAAAAAAAAAAAAAAAAAAAA==".into(),
                        },
                        width,
                        height,
                        scale,
                        format: "bgra8".into(),
                        input: Some(input),
                    })
                    .unwrap();
            }
        }
        if message == FromSidecar::End {
            return;
        }
    }
}

struct Host {
    child: Child,
    input: Option<ChildStdin>,
    lines: mpsc::Receiver<Value>,
}

impl Drop for Host {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Host {
    fn start() -> Host {
        let mut child = Command::new(env!("CARGO_BIN_EXE_soksak-app"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let input = child.stdin.take().unwrap();
        let output = child.stdout.take().unwrap();
        let (sender, lines) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(output).lines() {
                let line = line.unwrap();
                let value =
                    serde_json::from_str(&line).unwrap_or_else(|error| panic!("{error}: {line}"));
                if sender.send(value).is_err() {
                    return;
                }
            }
        });
        Host {
            child,
            input: Some(input),
            lines,
        }
    }

    /// Closes standard input, waits for the sidecar to end and returns its standard error.
    fn finish(&mut self) -> String {
        drop(self.input.take());
        assert!(self.child.wait().unwrap().success());
        let mut records = String::new();
        std::io::Read::read_to_string(&mut self.child.stderr.take().unwrap(), &mut records)
            .unwrap();
        records
    }

    fn send(&mut self, value: Value) {
        self.input
            .as_mut()
            .unwrap()
            .write_all(format!("{value}\n").as_bytes())
            .unwrap();
    }

    fn next(&self) -> Value {
        self.lines
            .recv_timeout(STEP)
            .expect("the sidecar sends a line within the step")
    }

    /// The next body of the surface, skipping nothing.
    fn body(&self) -> Value {
        let line = self.next();
        assert_eq!(line["surface"], "s1", "{line}");
        line["body"].clone()
    }

    fn open(&mut self, command: Vec<String>) {
        self.send(json!({"surface": "s1", "root": "/", "body": {"operation": "open", "command": command}}));
    }
}

fn application(role: &str) -> Vec<String> {
    vec![
        "/usr/bin/env".into(),
        format!("{ROLE}={role}"),
        std::env::current_exe().unwrap().display().to_string(),
        "--exact".into(),
        "fake_application".into(),
        "--nocapture".into(),
    ]
}

fn configure() -> Value {
    json!({"surface": "s1", "body": {"image": {"configure": {"name": "view", "generation": 2, "raster": 5, "width": 64, "height": 32, "scale": 2.0}}}})
}

#[test]
fn the_sidecar_relays_configure_frames_answers_input_and_close() {
    let mut host = Host::start();
    host.open(application("echo"));
    assert_eq!(host.body()["event"], "started");
    assert_eq!(
        host.body(),
        json!({"event": "connected", "toolkit": "fake"})
    );
    host.send(configure());
    assert_eq!(
        host.body(),
        json!({"event": "state", "title": "{\"type\":\"configure\",\"width\":64,\"height\":32,\"scale\":2.0}"})
    );
    assert_eq!(
        host.body(),
        json!({"image": {"name": "view", "generation": 2, "raster": 5, "token": {"kind": "iosurface-global", "id": 1, "nonce": "AAAAAAAAAAAAAAAAAAAAAA=="}, "width": 64, "height": 32, "scale": 2.0, "format": "bgra8", "sequence": 1}})
    );
    assert_eq!(host.body(), json!({"event": "cursor", "shape": "text"}));
    host.send(json!({"surface": "s1", "body": {"image": {"consumed": {"name": "view", "generation": 2, "raster": 5, "sequence": 1}}}}));
    assert_eq!(
        host.body()["title"],
        "{\"type\":\"consumed\",\"sequence\":1}"
    );
    let pointer = json!({"type": "pointer", "phase": "move", "x": 3.0, "y": 4.0, "buttons": [], "shift": false, "alt": false, "ctrl": false, "meta": false, "input": 11});
    host.send(json!({"surface": "s1", "body": {"operation": "input", "message": pointer}}));
    let title: Value = serde_json::from_str(host.body()["title"].as_str().unwrap()).unwrap();
    assert_eq!(title, pointer);
    assert_eq!(host.body()["image"]["sequence"], 2);
    assert_eq!(host.body()["image"]["sequence"], 3);
    host.send(json!({"surface": "s1", "root": "/", "closed": true}));
    assert_eq!(host.body()["title"], "{\"type\":\"end\"}");
    assert_eq!(
        host.body(),
        json!({"event": "exited", "exit": "exit status 0"})
    );
    assert_eq!(host.next(), json!({"surface": "s1", "closed": true}));
    let records = host.finish();
    let frames: Vec<&str> = records
        .lines()
        .filter(|line| line.starts_with("frame "))
        .collect();
    assert_eq!(
        frames,
        vec!["frame 2 input 11"],
        "only the first frame that shows a new input is recorded: {records}"
    );
}

#[test]
fn a_refused_frame_and_a_frame_of_another_size_free_their_buffers() {
    let mut host = Host::start();
    host.open(application("mismatch"));
    host.body();
    host.body();
    host.send(configure());
    host.body();
    let error = host.body();
    assert_eq!(
        error,
        json!({"error": "frame 1 is 65 by 32 at scale 2, but the region view is configured as 64 by 32 at scale 2"})
    );
    assert_eq!(host.body(), json!({"event": "cursor", "shape": "text"}));
    assert_eq!(
        host.body()["title"],
        "{\"type\":\"consumed\",\"sequence\":1}"
    );
    host.send(json!({"surface": "s1", "body": {"image": {"error": "notFound", "name": "view", "generation": 2, "raster": 5, "sequence": 1}}}));
    assert_eq!(
        host.body(),
        json!({"error": "the host refused frame 1 of the region view: notFound"})
    );
    assert_eq!(
        host.body()["title"],
        "{\"type\":\"consumed\",\"sequence\":1}"
    );
    host.send(json!({"surface": "s1", "body": {"image": {"error": "stale", "name": "view", "generation": 2, "raster": 5, "sequence": 1}}}));
    assert_eq!(
        host.body()["title"],
        "{\"type\":\"consumed\",\"sequence\":1}",
        "a stale frame is no error"
    );
    host.send(json!({"surface": "s1", "root": "/", "closed": true}));
    host.body();
    host.body();
    assert_eq!(host.next(), json!({"surface": "s1", "closed": true}));
}

#[test]
fn an_application_that_ends_by_itself_is_an_error_that_names_its_exit() {
    let mut host = Host::start();
    host.open(vec!["/usr/bin/false".into()]);
    assert_eq!(host.body()["event"], "started");
    assert_eq!(
        host.body(),
        json!({"event": "exited", "exit": "exit status 1"})
    );
    assert_eq!(
        host.body(),
        json!({"error": "the application /usr/bin/false ended by itself: exit status 1"})
    );
    let key = json!({"type": "key", "key": "a", "down": true, "shift": false, "alt": false, "ctrl": false, "meta": false, "repeat": false});
    host.send(json!({"surface": "s1", "body": {"operation": "input", "message": key}}));
    assert_eq!(
        host.body(),
        json!({"error": "input: the surface s1 has no running application"})
    );
    host.send(json!({"surface": "s1", "root": "/", "closed": true}));
    assert_eq!(host.next(), json!({"surface": "s1", "closed": true}));
}

#[test]
fn a_request_that_breaks_the_protocol_is_an_error_that_names_it() {
    let mut host = Host::start();
    host.send(json!({"surface": "s1", "root": "/", "body": {"operation": "open", "command": []}}));
    assert_eq!(
        host.body(),
        json!({"error": "open: command must name a program"})
    );
    host.send(json!({"surface": "s1", "root": "/", "body": {"operation": "teleport"}}));
    assert_eq!(
        host.body(),
        json!({"error": "unknown request {\"operation\":\"teleport\"}"})
    );
    host.open(application("echo"));
    host.body();
    host.body();
    let configure = json!({"type": "configure", "width": 1, "height": 1, "scale": 1});
    host.send(json!({"surface": "s1", "body": {"operation": "input", "message": configure}}));
    assert_eq!(
        host.body(),
        json!({"error": "input: configure is not an input message"})
    );
    host.send(json!({"surface": "s1", "root": "/", "closed": true}));
    assert_eq!(
        host.body(),
        json!({"event": "exited", "exit": "exit status 0"})
    );
    assert_eq!(host.next(), json!({"surface": "s1", "closed": true}));
}
