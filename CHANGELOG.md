# Changelog

[한국어](CHANGELOG.ko.md)

## Unreleased

- A1.1.1: the sidecar takes the embed kit from its published release v0.0.1 instead of a path to a local checkout.
- A1.9: a frame whose size or scale differs from the last configure is answered with `consumed` and recorded on standard error as `frame <sequence> stale: …`, and the page shows no error. Input: while a region was resized the page showed "frame 1200 is 1320 by 829 at scale 1, but the region view is configured as 1319 by 829", because the application draws at the size it had before the new configure reached it. Red: the page received that error. Green: 6 of 6.
- A1.8: a configure that arrives before the open of its surface is kept and passed to the application that the open starts. Input: the host configures the region when the page creates its composition, which can be before the open of the page, and the sidecar refused that configure with an error. Red: the configure was refused. Green: 6 of 6.
- A1.7: `package.json` names its repository, which the workspace registry reads for every release.
- A1.6: the sidecar records `frame <sequence> input <n>` on standard error for the first frame that shows a new input; the image envelope has no member for the input, so this record joins the input that the page sent to the answer of the host for the frame that shows it. Red: no record. Green: one record for two frames of one input.
- A1.5: `files` of `package.json` lists `sidecar.json` and `build/soksak-app`, so `sok sidecar release` packs the sidecar.
- A1.3, A1.4: the sidecar starts the application of a surface with the embed socket in its environment, no pipe of the host and its own process group; relays configure, frames as image envelopes, every answer of the host as `consumed` and the input of the page; passes cursor, caret, state and errors to the page; ends the application on `closed` with `end` and a 2-second stop; and reports an application that ends by itself, a frame of another size and a refused frame as errors that name them. Red: 4 of 5 tests failed against the empty sidecar. Green: 5 of 5. Both items are implemented in one change because one surface task serves both.
- A1.2: the protocol document states the requests, the event bodies and the image envelope of the sidecar, and passes every answer of the host to the application as `consumed`.
- A1.1: the application sidecar repository starts at version 0.0.1 with its build, format, lint, test and record checks, the commit and record hooks, a README, its checklist and this changelog.
