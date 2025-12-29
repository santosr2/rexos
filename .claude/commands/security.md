---
description: Run security audit on dependencies
allowed-tools: Bash(cargo:*), Read
---

Run comprehensive security checks on RexOS dependencies:

1. Check for known vulnerabilities:
```bash
cargo audit 2>&1
```

2. Check licenses and security advisories:
```bash
cargo deny check 2>&1
```

3. Find unused dependencies:
```bash
cargo machete 2>&1
```

Report any security issues or concerns found.
