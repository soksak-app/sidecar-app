# Features

[한국어](features.ko.md)

- [~] A1 — P1: The application sidecar at version 0.0.1.
  - [o] A1.1 — P1: The repository with its build, test and record checks, a README, a checklist and a changelog.
    - [ ] A1.1.1 — P2: Replace the path dependency on the embed kit with its published release once the kit repository is published.
  - [ ] A1.2 — P1: The protocol document of the sidecar: its requests, its event bodies and its image envelope.
  - [ ] A1.3 — P1: Start the application with the embed socket in its environment, stop it when the surface closes, and report an application that ends as an error that names it.
  - [ ] A1.4 — P1: Relay the configure of the host to the application, each frame of the application to the host as an image envelope, each answer of the host as `consumed`, and the input of the page to the application.
