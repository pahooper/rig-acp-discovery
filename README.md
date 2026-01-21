# rig-acp-discovery

Agent discovery library for AI coding agents (Claude Code, Codex, OpenCode, Gemini).

Part of the [rig-acp](https://github.com/pahooper/rig-acp) project.

## Features

- **Detection**: Detect installed AI agent CLIs on Linux and Windows
- **Version Parsing**: Parse agent versions into semver format
- **Installation Info**: Get platform-appropriate install commands
- **Programmatic Installation**: Trigger agent installation with progress callbacks

## Usage

```rust
use rig_acp_discovery::{AgentKind, detect, detect_all};

// Detect a specific agent
let status = detect(AgentKind::ClaudeCode).await;

// Detect all agents in parallel
let results = detect_all().await;

// Get installation info
let info = AgentKind::Codex.install_info();
```

## License

MIT OR Apache-2.0
