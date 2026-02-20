# Local Agent Presets

This directory provides starter templates for optional local coding-agent guidance.

## Usage

Copy this folder to `.agent` in the project root:

```bash
cp -R .agent.example .agent
```

You can then edit `.agent/` for your machine and workflow.

## Notes

- `.agent/` is intentionally ignored by git.
- Keep secrets and machine-specific values only in `.agent/`, never in `.agent.example/`.
- If you do not use coding agents, you can skip this setup.
