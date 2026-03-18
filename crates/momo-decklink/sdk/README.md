# DeckLink SDK 15.3

This directory contains files from the Blackmagic DeckLink SDK.

## License

Per the SDK's End User License Agreement (Section 0), the Include headers
(`/Linux/Include`, `/Win/Include`, `/Mac/Include`) are exempt from the
redistribution restrictions (Clauses 1, 4.3, 4.4, 5, 7, 8). These headers
are committed to git and may be freely copied, modified, and redistributed.

All other SDK files (samples, documentation, binaries) are subject to the
full EULA and are excluded from git via `.gitignore`.

## Structure

```
sdk/
├── include/          # C++ headers — committed to git (EULA Section 0 exempt)
├── samples/          # Official C++ samples — NOT committed (full EULA applies)
├── examples/         # Cross-platform examples — NOT committed
└── *.pdf             # SDK docs and EULA — NOT committed
```

## Updating the SDK

1. Go to https://www.blackmagicdesign.com/developer
2. Download "Desktop Video SDK"
3. Extract `Linux/include/*` into `include/` and commit
4. Extract `Linux/Samples/*` into `samples/` (optional, local only)
