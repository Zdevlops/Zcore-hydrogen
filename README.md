# Z-Core Bench (Hybrid Rust + Python Android App)
**BY @Zdevlops**

## Struktur Repository GitHub
```text
├── .github/
│   └── workflows/
│       ├── build.yml       # Auto-compile Rust .so + Buildozer Android APK
│       └── rapikan.yml     # Auto-rapikan struktur folder saat push
├── rust_core/
│   ├── Cargo.toml
│   ├── pyproject.toml
│   └── src/
│       └── lib.rs          # Rust PyO3 Benchmark, FPS 1%/0.1% Low & Telemetry Engine
├── app/
│   ├── main.py             # KivyMD UI (Benchmark, Optimize, Permissions, Info)
│   ├── assets/             # Output libzcore_bench_engine.so
│   └── android_bridge/
│       ├── __init__.py
│       ├── permissions.py
│       ├── shizuku_bridge.py
│       ├── device_advisor.py
│       └── overlay_service.py
├── buildozer.spec
└── requirements.txt
```
