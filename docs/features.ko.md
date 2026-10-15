# Features

[English](features.md)

- [~] A1 — P1: 버전 0.0.1의 애플리케이션 sidecar.
  - [o] A1.1 — P1: build·시험·기록 검사, README, 체크리스트, 체인지로그를 가진 저장소.
    - [ ] A1.1.1 — P2: embed kit 저장소가 공개되면 embed kit의 path 의존을 공개된 release로 바꾼다.
  - [o] A1.2 — P1: sidecar의 protocol 문서: 요청, event body, image envelope.
  - [o] A1.3 — P1: embed socket을 환경에 넣어 애플리케이션을 시작하고, 표면이 닫히면 멈추며, 끝난 애플리케이션을 그것을 이름하는 오류로 보고한다.
  - [o] A1.4 — P1: host의 configure를 애플리케이션에, 애플리케이션의 각 frame을 image envelope로 host에, host의 각 응답을 `consumed`로, page의 입력을 애플리케이션에 중계한다.
  - [o] A1.5 — P1: release가 `package.json`의 `files`에 `sidecar.json`과 build한 실행 파일을 두어 `sok sidecar release`가 그것들을 묶는다.
  - [o] A1.6 — P1: sidecar는 새 입력을 처음 보이는 frame에 `frame <sequence> input <n>`을 standard error에 기록해, page의 입력을 그 frame에 대한 host의 응답에 잇는다.
