# Jon

**Natural language interface for the Joyint ecosystem.**

Jon is a CLI and chat interface that translates between human language and Joy/Jyn commands. It has no data layer of its own - it calls Joy and Jyn under the hood and formats the results. Part of the [Joyint](https://github.com/joyint) ecosystem.

## Quick Start

```sh
cargo install jon-cli

jon "what's my next task?"           # -> jyn ls --sort=priority --limit=1
jon "move JOY-0045 to review"        # -> joy status JOY-0045 review
jon "how much did AI cost this week?" # -> joy costs --since=monday
```

## Features

- **Three tiers of intelligence** - pattern router (offline, instant), local mini-LLM (offline, opt-in), external LLM (API key or Joyint Pro)
- **Single binary** - one small Rust binary, no runtime dependencies
- **Offline-first** - Tier 0 pattern router works without any network or LLM
- **No data layer** - Jon owns nothing, all project data lives in Joy and Jyn

## Documentation

- [Vision](docs/dev/vision/README.md) - Product goals and design decisions

For architecture and contributing guidelines see `joy ai init` after setup.

## Status

Pre-development. See the [Vision](docs/dev/vision/README.md) for details.

## License

MIT
