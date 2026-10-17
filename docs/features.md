# Features

[한국어](features.ko.md)

- [~] A1 — P1: The application sidecar at version 0.0.1.
  - [o] A1.1 — P1: The repository with its build, test and record checks, a README, a checklist and a changelog.
    - [o] A1.1.1 — P2: Replace the path dependency on the embed kit with its published release once the kit repository is published.
  - [o] A1.2 — P1: The protocol document of the sidecar: its requests, its event bodies and its image envelope.
  - [o] A1.3 — P1: Start the application with the embed socket in its environment, stop it when the surface closes, and report an application that ends as an error that names it.
  - [o] A1.4 — P1: Relay the configure of the host to the application, each frame of the application to the host as an image envelope, each answer of the host as `consumed`, and the input of the page to the application.
  - [o] A1.5 — P1: The release lists `sidecar.json` and the built executable in `files` of `package.json`, so `sok sidecar release` packs them.
  - [o] A1.6 — P1: The sidecar records `frame <sequence> input <n>` on standard error for the first frame that shows a new input, which joins the input of the page to the answer of the host for that frame.
  - [o] A1.7 — P1: `package.json` names its repository, which the workspace registry requires.
  - [o] A1.8 — P1: A configure that arrives before the open of its surface is kept and passed to the application that the open starts.
  - [o] A1.9 — P1: A frame of an earlier configure is answered with `consumed` and recorded, not reported to the page: while a region changes size the application draws frames at the size it had before the new configure reached it.
  - [o] A1.10 — P1: A frame that the host answers with `notAttached`, for a region that is not attached in the current composition or generation, frees its buffer and is recorded, not reported to the page, as a `stale` answer is: the page opens the application before it attaches the region, so the first frames can arrive before the attachment, and the connector window check of core reported the error intermittently.
- [o] A2 — P2: Release path: `make release OUT=<folder>` makes the registry release with sok of core, the release workflow publishes it for a tag, and the ci workflow runs `make test` and the commit message check.
- [o] A3 — P1: Retired: the external application surface of core is retired, so nothing uses this repository; it is archived.
