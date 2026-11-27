# Architecture

Alpinoxi is a native Rust application split by responsibility:

```
alpinoxi_client
    ├── polished desktop UI
    ├── launcher state
    └── user actions
          │
          ├── alpinoxi_core
          │     ├── account profiles
          │     ├── RAM/launch policies
          │     └── installed-instance metadata
          │
          └── alpinoxi_auth_system
                ├── provider selection
                ├── OAuth flow state
                └── token-storage boundary
```

The UI never receives an account password or raw refresh token. Provider implementations will return an opaque account profile only after a successful official flow. Secure storage is platform-specific work for the authentication milestone.

## Memory policy

The launcher must not guess an unlimited heap. It presents explicit per-instance choices and validates a custom limit before launch. These are Java game-process settings; they do not describe launcher memory usage.
