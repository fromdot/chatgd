# chatgd

경량 채팅 게이트웨이 데몬 — 메시징 플랫폼과 CLI 도구를 연결합니다.

텔레그램 메시지를 받아 Claude Code, Ollama 등의 CLI 도구를 서브프로세스로 실행하고, 결과를 다시 채팅으로 전달합니다.

## 특징

- **텔레그램 → CLI 브릿지**: 메시지를 받아 설정된 백엔드 명령어 실행
- **다중 백엔드**: Claude, Ollama 등 여러 CLI 도구를 트리거로 전환
- **세션 관리**: 채팅별 대화 컨텍스트 유지 (JSONL 로그)
- **보안**: 허용된 사용자만 접근 가능 (`allowed_users`)

## 설치

### GitHub Release (권장)

[Releases](https://github.com/fromdot/chatgd/releases)에서 플랫폼에 맞는 바이너리를 다운로드합니다.

| 파일명 | 플랫폼 |
|--------|--------|
| `chatgd-aarch64-linux` | Raspberry Pi 4 / ARM64 Linux |
| `chatgd-x86_64-linux` | x86_64 Linux |
| `chatgd-aarch64-macos` | Apple Silicon Mac |

```bash
# 예시: Raspberry Pi 4
curl -LO https://github.com/fromdot/chatgd/releases/latest/download/chatgd-aarch64-linux
chmod +x chatgd-aarch64-linux
mv chatgd-aarch64-linux ~/.local/bin/chatgd
```

### 소스에서 빌드

```bash
cd chatgd
cargo build --release
# 바이너리: target/release/chatgd
```

## 설정

`config.example.toml`을 복사하여 `config.toml`을 만듭니다.

```bash
cp config.example.toml config.toml
```

```toml
[telegram]
token = "YOUR_BOT_TOKEN_HERE"

[security]
# 비어있으면 모든 사용자 허용
allowed_users = [123456789]

[[backends]]
name = "claude"
command = ["claude", "--continue", "-p", "{prompt}", "--output-format", "json"]
trigger = "@claude"
default = true

[[backends]]
name = "ollama"
command = ["ollama", "run", "llama3", "{prompt}"]
trigger = "@llama"
```

### 텔레그램 봇 토큰

1. [@BotFather](https://t.me/BotFather)에서 봇 생성
2. 발급된 토큰을 `config.toml`에 입력하거나 환경변수로 설정:

```bash
export CHATGD_TELEGRAM_TOKEN="your_token_here"
```

> 환경변수 `CHATGD_TELEGRAM_TOKEN`이 설정되면 `config.toml`의 값을 덮어씁니다.

## 사용법

```bash
# config.toml이 있는 디렉토리에서 실행
chatgd
```

### 텔레그램에서

- `@봇이름 질문 내용` — 기본 백엔드(claude)로 질문
- `@봇이름 @llama 질문 내용` — 특정 백엔드 지정
- `@봇이름 /reset` — 세션 초기화
- `@봇이름 /status` — 현재 상태 확인

### 백엔드 추가

`config.toml`의 `[[backends]]` 섹션을 추가하면 됩니다. `command`의 `{prompt}`가 사용자 입력으로 치환됩니다.

```toml
[[backends]]
name = "my-tool"
command = ["my-cli", "--query", "{prompt}"]
trigger = "@mytool"
```

## 구조

```
chatgd/
├── src/
│   ├── main.rs              # 진입점, 설정 로드
│   ├── adapter/
│   │   ├── mod.rs
│   │   └── telegram.rs      # 텔레그램 폴링 + 메시지 핸들링
│   ├── backend/
│   │   ├── mod.rs            # 백엔드 설정, 서브커맨드 파싱
│   │   └── subprocess.rs     # CLI 서브프로세스 실행
│   └── session.rs            # 세션/로그 관리
├── config.example.toml
└── Cargo.toml
```

## 라이선스

MIT
