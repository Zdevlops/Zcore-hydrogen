[app]
title = Z-Core Bench
package.name = zcorebench
package.domain = org.zdevlops
source.dir = app
source.include_exts = py,png,jpg,kv,atlas,so,json
version = 1.0.0
requirements = python3,kivy==2.3.0,kivymd==1.2.0,pyjnius
orientation = portrait
fullscreen = 0

# Android Permissions required by PRD
android.permissions = SYSTEM_ALERT_WINDOW,REQUEST_IGNORE_BATTERY_OPTIMIZATIONS,PACKAGE_USAGE_STATS,FOREGROUND_SERVICE,INTERNET,ACCESS_NETWORK_STATE
android.api = 34
android.minapi = 26
android.ndk = 25b
android.archs = arm64-v8a
android.allow_backup = True
android.add_libs_arm64_v8a = app/assets/libzcore_bench_engine.so

[buildozer]
log_level = 2
warn_on_root = 0
