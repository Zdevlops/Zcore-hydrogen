class FloatingOverlayController:
    """
    Manages the SYSTEM_ALERT_WINDOW compact floating HUD displaying real-time
    Device Temperature (°C), CPU Usage (%), and GPU Usage (%) while gaming.
    """

    def __init__(self):
        self.enabled = False

    def toggle(self, has_overlay_permission: bool) -> bool:
        if not has_overlay_permission:
            self.enabled = False
            return False
        self.enabled = not self.enabled
        return self.enabled
