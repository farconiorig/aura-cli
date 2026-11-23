# Aura CLI — Dota 2 Configuration Optimizer (Community Edition)

Aura CLI is an open-source tool for optimizing Dota 2 settings (`video.txt`, `autoexec.cfg`).

## Capabilities
- 100% VAC-Safe: Config-only optimization without memory modifications
- Pre-configured competitive frametime presets (competitive, balanced, potato)
- Automated backup creation and one-command rollback

## Usage
```bash
# Check detected Dota 2 environment
aura-cli --status

# Apply competitive preset
aura-cli --apply competitive

# Clean shader cache
aura-cli --clean-cache

# Rollback configurations to backup
aura-cli --restore
```

## Building
```bash
cargo build --release
```
