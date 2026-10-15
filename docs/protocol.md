# Protocol

[한국어](protocol.ko.md)

The application sidecar runs one application per surface. The application speaks the embed protocol of the embed kit on a Unix socket that the sidecar names in its environment; the sidecar speaks the sidecar protocol of soksak with the host on standard input and output.

## Requests

| Body | Effect |
|---|---|
| `{"operation": "open", "command": [program, argument, …]}` | Starts `program` with the arguments and `SOKSAK_EMBED_SOCKET` in its environment. An `open` while the application of the surface runs with the same command does nothing; another command is an error. |
| `{"operation": "input", "message": message}` | Passes `message` to the application. It is a message of the embed protocol from the sidecar of the type `key`, `compose`, `commit`, `pointer`, `scroll` or `focus`; another type, a message that breaks the embed protocol, or a surface without a connected application is an error. |
| `{"image": {"configure": {name, generation, raster, width, height, scale}}}` | The host configured the region that the sidecar supplies. The sidecar passes `configure` with the size and scale to the application and uses the name, generation and raster for the frames after it. A configure before the open of the surface is kept for the application that the open starts. |
| `{"image": {"consumed": {…}}}` or `{"image": {"error": kind, …}}` | The host finished with the frame of the sequence. The sidecar passes `consumed` with the sequence to the application in both cases, because the host reads the buffer of a refused frame no more; an error other than `stale` is also sent to the page as an error. |
| `closed` | Sends `end` to the application, ends it when it has not ended 2 seconds later, removes its socket and answers `closed`. |

## Event bodies

| Body | Meaning |
|---|---|
| `{"event": "started", "pid": n}` | The application started. |
| `{"event": "connected", "toolkit": text}` | The application connected and said `hello`. |
| `{"event": "cursor", "shape": shape}` | The cursor over the region, as the embed message `cursor` names it. |
| `{"event": "caret", "x", "y", "width", "height"}` | The caret of the input method in points of the region. |
| `{"event": "state", "title": text}` | The state of the application. |
| `{"event": "exited", "exit": text}` | The application ended; `exit` is `exit status <code>` or `signal <number>`. An end that the sidecar did not cause is also sent as an error. |
| `{"error": text}` | A request failed, the application failed, or the host refused a frame. |

## Image envelope

Each `frame` of the application becomes the image envelope of the sidecar protocol: the name, generation and raster of the last configure, and the token, width, height, scale, format and sequence of the frame. A frame before the first configure, or a frame whose size or scale differs from the last configure, is an error of the application, and the sidecar answers it with `consumed` so that its buffer is free again.

## Records

The sidecar writes `frame <sequence> input <n>` on standard error for the first frame that shows a new input, which the host stores as a record of the sidecar. The image envelope has no member for the input, so this record joins the input that the page sent to the answer of the host for the frame that shows it.
