<div align="center">

# 💬 chatgd

**경량 채팅 게이트웨이 데몬**

메시징 플랫폼과 CLI 도구를 연결합니다.

[![CI](https://github.com/fromdot/chatgd/actions/workflows/ci.yml/badge.svg)](https://github.com/fromdot/chatgd/actions/workflows/ci.yml)
[![Release](https://github.com/fromdot/chatgd/actions/workflows/release.yml/badge.svg)](https://github.com/fromdot/chatgd/actions/workflows/release.yml)
[![GitHub Release](https://img.shields.io/github/v/release/fromdot/chatgd?style=flat-square&color=blue)](https://github.com/fromdot/chatgd/releases/latest)
[![License: MIT](https://img.shields.io/badge/license-MIT-green?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2024_edition-orange?style=flat-square&logo=rust)](https://www.rust-lang.org)

<br/>

<img src="https://img.shields.io/badge/Telegram-Bot-26A5E4?style=for-the-badge&logo=telegram&logoColor=white" alt="Telegram"/>
<img src="https://img.shields.io/badge/Claude-CLI-6B4FBB?style=for-the-badge&logo=anthropic&logoColor=white" alt="Claude"/>
<img src="https://img.shields.io/badge/Ollama-Local_LLM-333333?style=for-the-badge&logo=ollama&logoColor=white" alt="Ollama"/>

</div>

---

텔레그램 메시지를 받아 **Claude Code**, **Ollama** 등의 CLI 도구를 서브프로세스로 실행하고, 결과를 다시 채팅으로 전달합니다.

## ✨ 특징

| 기능 | 설명 |
|------|------|
| 🔗 **플랫폼 브릿지** | 텔레그램 메시지 → CLI 도구 실행 → 응답 전달 |
| 🔀 **다중 백엔드** | `@claude`, `@llama` 등 트리거로 백엔드 전환 |
| 💾 **세션 관리** | 채팅별 대화 컨텍스트 유지 (JSONL 로그) |
| 🔒 **접근 제어** | `allowed_users`로 허용된 사용자만 접근 |
| 🦀 **순수 Rust TLS** | `rustls` 사용 — 네이티브 라이브러리 의존 없음 |
| 🏗️ **크로스 컴파일** | ARM64 Linux (RPi4), x86_64 Linux, Apple Silicon 지원 |

## 📦 설치

### GitHub Release (권장)

[**→ 최신 릴리즈 다운로드**](https://github.com/fromdot/chatgd/releases/latest)

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

## ⚙️ 설정

`config.example.toml`을 복사하여 `config.toml`을 만듭니다.

```bash
cp chatgd/config.example.toml config.toml
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

<details>
<summary><b>🔑 텔레그램 봇 토큰 설정</b></summary>

1. [@BotFather](https://t.me/BotFather)에서 봇 생성
2. 발급된 토큰을 `config.toml`에 입력하거나 환경변수로 설정:

```bash
export CHATGD_TELEGRAM_TOKEN="your_token_here"
```

> 환경변수 `CHATGD_TELEGRAM_TOKEN`이 설정되면 `config.toml`의 값을 덮어씁니다.

</details>

## 🚀 사용법

```bash
# config.toml이 있는 디렉토리에서 실행
chatgd
```

### 텔레그램 명령어

| 명령어 | 설명 |
|--------|------|
| `@봇이름 질문 내용` | 기본 백엔드(claude)로 질문 |
| `@봇이름 @llama 질문 내용` | 특정 백엔드 지정 |
| `@봇이름 /reset` | 세션 초기화 |
| `@봇이름 /status` | 현재 상태 확인 |

### 백엔드 추가

`config.toml`의 `[[backends]]` 섹션을 추가합니다. `{prompt}`가 사용자 입력으로 치환됩니다.

```toml
[[backends]]
name = "my-tool"
command = ["my-cli", "--query", "{prompt}"]
trigger = "@mytool"
```

## 🏛️ 구조

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

## 🗺️ 로드맵

- [x] 텔레그램 어댑터
- [x] 다중 백엔드 (Claude, Ollama)
- [x] 세션 컨텍스트 관리
- [x] GitHub Actions CI/CD + 크로스 컴파일
- [ ] 카카오톡 채널 어댑터 (Kakao i Open Builder)
- [ ] 캘린더 연동 (Kakao Calendar API)

## 📄 라이선스

[MIT](LICENSE)

---

<div align="center">
<sub>Built with 🦀 Rust · Deployed on 🍓 Raspberry Pi</sub>
</div>
