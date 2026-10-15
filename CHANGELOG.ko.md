# Changelog

[English](CHANGELOG.md)

## Unreleased

- A1.8: 표면의 open보다 먼저 온 configure를 보관했다가 open이 시작한 애플리케이션에 넘긴다. 입력: host는 page가 composition을 만들 때 region을 configure하며 그것은 page의 open보다 앞설 수 있는데, sidecar가 그 configure를 오류로 거부했다. Red: configure가 거부되었다. Green: 6개 중 6개.
- A1.7: workspace registry가 모든 release에서 읽는 저장소를 `package.json`이 이름한다.
- A1.6: sidecar는 새 입력을 처음 보이는 frame에 `frame <sequence> input <n>`을 standard error에 기록한다. image envelope에는 입력의 member가 없으므로, 이 기록이 page가 보낸 입력을 그것을 보이는 frame에 대한 host의 응답에 잇는다. Red: 기록 없음. Green: 한 입력의 frame 두 개에 기록 하나.
- A1.5: `package.json`의 `files`가 `sidecar.json`과 `build/soksak-app`을 두어 `sok sidecar release`가 sidecar를 묶는다.
- A1.3, A1.4: sidecar가 표면의 애플리케이션을 환경의 embed socket과 함께, host의 pipe 없이, 자기 process group으로 시작한다. configure, image envelope로 바꾼 frame, `consumed`로 바꾼 host의 모든 응답, page의 입력을 중계하고, cursor, caret, 상태, 오류를 page에 넘긴다. `closed`에는 `end`와 2초 정지로 애플리케이션을 끝내고, 스스로 끝난 애플리케이션, 다른 크기의 frame, 거부된 frame을 그것을 이름하는 오류로 보고한다. Red: 빈 sidecar에서 시험 5개 중 4개가 실패했다. Green: 5개 중 5개. 표면 task 하나가 두 항목을 맡으므로 한 변경으로 구현했다.
- A1.2: protocol 문서가 sidecar의 요청, event body, image envelope를 정하고, host의 모든 응답을 애플리케이션에 `consumed`로 넘긴다.
- A1.1: 애플리케이션 sidecar 저장소가 build·format·lint·시험·기록 검사, commit과 기록 hook, README, 체크리스트, 이 체인지로그를 가지고 버전 0.0.1로 시작한다.
