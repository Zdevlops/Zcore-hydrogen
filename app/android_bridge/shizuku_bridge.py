import subprocess


class ShizukuFPSController:
    """
    Verifies Shizuku Binder / Wireless ADB privilege state and dispatches
    privileged frame-rate lock commands (`cmd game set --fps` + `peak_refresh_rate`).
    """

    SUPPORTED_RATES = ["30 FPS", "45 FPS", "60 FPS", "90 FPS", "120 FPS", "Unlocked"]

    @staticmethod
    def check_privilege_active() -> dict:
        # Check Shizuku rish/service or local Wireless ADB daemon state
        try:
            res = subprocess.run(
                ["sh", "-c", "service check shizuku || getprop service.adb.tcp.port"],
                capture_output=True,
                text=True,
                timeout=1.5,
            )
            out = res.stdout.strip()
            if "found" in out.lower() or (out.isdigit() and int(out) > 0):
                return {"active": True, "mode": "Shizuku / Wireless ADB Active"}
        except Exception:
            pass
        return {"active": False, "mode": "Not Activated"}

    @classmethod
    def apply_fps_lock(cls, target_label: str, package_name: str) -> dict:
        status = cls.check_privilege_active()
        if not status["active"]:
            return {
                "ok": False,
                "message": "Requires Shizuku or Wireless Debugging activation first.",
            }

        if target_label == "Unlocked":
            cmds = [
                f"cmd game reset {package_name}",
                "settings put system peak_refresh_rate 120.0",
                "settings put system min_refresh_rate 0.0",
            ]
        else:
            fps_val = int(target_label.split()[0])
            cmds = [
                f"cmd game set --fps {fps_val} {package_name}",
                f"settings put system peak_refresh_rate {float(fps_val)}",
            ]

        for cmd in cmds:
            try:
                subprocess.run(["sh", "-c", cmd], timeout=2.0, check=False)
            except Exception:
                pass

        return {"ok": True, "message": f"Applied {target_label} on {package_name}"}
