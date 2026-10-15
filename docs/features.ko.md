# Features

[English](features.md)

- [~] A1 — P1: 버전 0.0.1의 애플리케이션 sidecar.
  - [o] A1.1 — P1: build·시험·기록 검사, README, 체크리스트, 체인지로그를 가진 저장소.
    - [o] A1.1.1 — P2: embed kit 저장소가 공개되면 embed kit의 path 의존을 공개된 release로 바꾼다.
  - [o] A1.2 — P1: sidecar의 protocol 문서: 요청, event body, image envelope.
  - [o] A1.3 — P1: embed socket을 환경에 넣어 애플리케이션을 시작하고, 표면이 닫히면 멈추며, 끝난 애플리케이션을 그것을 이름하는 오류로 보고한다.
  - [o] A1.4 — P1: host의 configure를 애플리케이션에, 애플리케이션의 각 frame을 image envelope로 host에, host의 각 응답을 `consumed`로, page의 입력을 애플리케이션에 중계한다.
  - [o] A1.5 — P1: release가 `package.json`의 `files`에 `sidecar.json`과 build한 실행 파일을 두어 `sok sidecar release`가 그것들을 묶는다.
  - [o] A1.6 — P1: sidecar는 새 입력을 처음 보이는 frame에 `frame <sequence> input <n>`을 standard error에 기록해, page의 입력을 그 frame에 대한 host의 응답에 잇는다.
  - [o] A1.7 — P1: workspace registry가 요구하는 저장소를 `package.json`이 이름한다.
  - [o] A1.8 — P1: 표면의 open보다 먼저 온 configure를 보관했다가 open이 시작한 애플리케이션에 넘긴다.
  - [o] A1.9 — P1: 이전 configure의 frame은 page에 오류로 보고하지 않고 `consumed`로 답하며 기록한다: region 크기가 바뀌는 동안 애플리케이션은 새 configure가 닿기 전의 크기로 frame을 그린다.
  - [o] A1.10 — P1: host가 현재 composition이나 generation에 붙지 않은 region 때문에 `notAttached`로 답한 frame은, `stale` 답처럼 buffer를 풀고 기록되며 page에 보고되지 않는다. page는 region을 붙이기 전에 애플리케이션을 열므로 첫 frame이 붙기 전에 올 수 있고, core의 연결 창 검사가 그 오류를 간헐적으로 보고했다.
