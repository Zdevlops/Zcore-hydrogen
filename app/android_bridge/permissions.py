import os

try:
    from jnius import autoclass, cast
    PythonActivity = autoclass("org.kivy.android.PythonActivity")
    Settings = autoclass("android.provider.Settings")
    Uri = autoclass("android.net.Uri")
    Intent = autoclass("android.content.Intent")
    Context = autoclass("android.content.Context")
    AppOpsManager = autoclass("android.app.AppOpsManager")
    Process = autoclass("android.os.Process")
    ANDROID_AVAILABLE = True
except Exception:
    ANDROID_AVAILABLE = False


class PermissionManager:
    """Checks and requests SYSTEM_ALERT_WINDOW, Battery Optimization, and Usage Access."""

    @staticmethod
    def check_all() -> dict:
        if not ANDROID_AVAILABLE:
            return {
                "overlay": False,
                "battery": False,
                "usage": False,
            }
        activity = PythonActivity.mActivity
        ctx = activity.getApplicationContext()
        pkg_name = ctx.getPackageName()

        # 1. SYSTEM_ALERT_WINDOW
        can_overlay = bool(Settings.canDrawOverlays(ctx))

        # 2. REQUEST_IGNORE_BATTERY_OPTIMIZATIONS
        pm = cast("android.os.PowerManager", ctx.getSystemService(Context.POWER_SERVICE))
        ignoring_battery = bool(pm.isIgnoringBatteryOptimizations(pkg_name))

        # 3. PACKAGE_USAGE_STATS
        app_ops = cast("android.app.AppOpsManager", ctx.getSystemService(Context.APP_OPS_SERVICE))
        mode = app_ops.checkOpNoThrow(
            AppOpsManager.OPSTR_GET_USAGE_STATS,
            Process.myUid(),
            pkg_name
        )
        has_usage = (mode == AppOpsManager.MODE_ALLOWED)

        return {
            "overlay": can_overlay,
            "battery": ignoring_battery,
            "usage": has_usage,
        }

    @staticmethod
    def open_overlay_settings():
        if not ANDROID_AVAILABLE:
            return
        activity = PythonActivity.mActivity
        uri = Uri.parse(f"package:{activity.getPackageName()}")
        intent = Intent(Settings.ACTION_MANAGE_OVERLAY_PERMISSION, uri)
        activity.startActivity(intent)

    @staticmethod
    def open_battery_settings():
        if not ANDROID_AVAILABLE:
            return
        activity = PythonActivity.mActivity
        uri = Uri.parse(f"package:{activity.getPackageName()}")
        intent = Intent(Settings.ACTION_REQUEST_IGNORE_BATTERY_OPTIMIZATIONS, uri)
        activity.startActivity(intent)

    @staticmethod
    def open_usage_settings():
        if not ANDROID_AVAILABLE:
            return
        activity = PythonActivity.mActivity
        intent = Intent(Settings.ACTION_USAGE_ACCESS_SETTINGS)
        activity.startActivity(intent)
