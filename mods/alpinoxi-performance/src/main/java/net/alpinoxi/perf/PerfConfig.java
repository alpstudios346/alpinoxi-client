package net.alpinoxi.perf;

/**
 * The mod's few switches.
 *
 * <p>Kept in memory and mirrored to the launcher's settings file through the
 * bridge, so there is no second config format to learn.
 */
public final class PerfConfig {
    private static boolean hud = true;
    private static boolean governor = false;
    private static int targetFps = 120;

    private PerfConfig() {
    }

    public static boolean showHud() {
        return hud;
    }

    public static void setHud(boolean value) {
        hud = value;
    }

    public static boolean governorEnabled() {
        return governor;
    }

    public static void setGovernor(boolean value) {
        governor = value;
    }

    public static int targetFps() {
        return targetFps;
    }

    public static void setTargetFps(int value) {
        targetFps = Math.max(30, Math.min(480, value));
    }
}
