# Protocol

[English](protocol.md)

애플리케이션 sidecar는 표면마다 애플리케이션 하나를 실행한다. 애플리케이션은 sidecar가 환경에 이름한 Unix socket에서 embed kit의 embed protocol을 말하고, sidecar는 standard input·output에서 host와 soksak의 sidecar protocol을 말한다.

## 요청

| Body | 효과 |
|---|---|
| `{"operation": "open", "command": [program, argument, …]}` | 인자와 환경의 `SOKSAK_EMBED_SOCKET`을 주어 `program`을 시작한다. 표면의 애플리케이션이 같은 command로 실행 중일 때의 `open`은 아무것도 하지 않고, 다른 command는 오류다. |
| `{"operation": "input", "message": message}` | `message`를 애플리케이션에 넘긴다. 이것은 type이 `key`, `compose`, `commit`, `pointer`, `scroll`, `focus`인, sidecar에서 가는 embed protocol 메시지다. 다른 type, embed protocol을 어긴 메시지, 접속한 애플리케이션이 없는 표면은 오류다. |
| `{"image": {"configure": {name, generation, raster, width, height, scale}}}` | host가 sidecar가 공급하는 region을 configure했다. sidecar는 크기와 배율을 가진 `configure`를 애플리케이션에 넘기고, 이름, generation, raster를 그 뒤의 frame에 쓴다. |
| `{"image": {"consumed": {…}}}` 또는 `{"image": {"error": kind, …}}` | host가 그 sequence의 frame을 끝냈다. host는 거부한 frame의 버퍼도 더 읽지 않으므로 sidecar는 두 경우 모두 sequence를 가진 `consumed`를 애플리케이션에 넘긴다. `stale`이 아닌 오류는 page에도 오류로 보낸다. |
| `closed` | 애플리케이션에 `end`를 보내고, 2초 뒤에도 끝나지 않으면 끝내며, socket을 지우고 `closed`로 답한다. |

## Event body

| Body | 뜻 |
|---|---|
| `{"event": "started", "pid": n}` | 애플리케이션이 시작했다. |
| `{"event": "connected", "toolkit": text}` | 애플리케이션이 접속해 `hello`를 말했다. |
| `{"event": "cursor", "shape": shape}` | embed 메시지 `cursor`가 이름하는 region 위의 cursor. |
| `{"event": "caret", "x", "y", "width", "height"}` | region의 point로 나타낸 input method의 caret. |
| `{"event": "state", "title": text}` | 애플리케이션의 상태. |
| `{"event": "exited", "exit": text}` | 애플리케이션이 끝났다. `exit`는 `exit status <code>`나 `signal <number>`다. sidecar가 일으키지 않은 끝은 오류로도 보낸다. |
| `{"error": text}` | 요청이 실패했거나, 애플리케이션이 실패했거나, host가 frame을 거부했다. |

## Image envelope

애플리케이션의 각 `frame`은 sidecar protocol의 image envelope가 된다: 마지막 configure의 이름, generation, raster와 frame의 token, width, height, scale, format, sequence. 첫 configure 앞의 frame이나 크기 또는 배율이 마지막 configure와 다른 frame은 애플리케이션의 오류이고, sidecar는 그 버퍼가 다시 free가 되도록 `consumed`로 답한다.

## 기록

sidecar는 새 입력을 처음 보이는 frame에 `frame <sequence> input <n>`을 standard error에 쓰고, host는 그것을 sidecar의 기록으로 저장한다. image envelope에는 입력의 member가 없으므로, 이 기록이 page가 보낸 입력을 그것을 보이는 frame에 대한 host의 응답에 잇는다.
