package net.alpinoxi.bridge;

import java.util.Locale;

/**
 * What the frame numbers mean, in the launcher's own words.
 *
 * <p>The rules are small and deliberate: a client short of heap is told what to
 * change, and a client that is fine is told nothing, because advice that fires
 * on a healthy game is noise the player learns to ignore.
 */
public final class PerformanceRules {
    /** Below this, the game spends its time collecting instead of drawing. */
    private static final int HEAP_TIGHT_MIB = 2048;
    /** Above this, the extra heap only costs start-up time. */
    private static final int HEAP_ROOMY_MIB = 8192;

    private PerformanceRules() {
    }

    public static String report(int heapMib, String installedModsCsv) {
        String advice;
        String pressure;
        if (heapMib < HEAP_TIGHT_MIB) {
            pressure = "high";
            advice = "Heap is " + heapMib + " MB: raise it to at least " + HEAP_TIGHT_MIB
                    + " MB in Alpinoxi's memory settings.";
        } else if (heapMib > HEAP_ROOMY_MIB) {
            pressure = "low";
            advice = "Heap is " + heapMib + " MB: more than the game needs. "
                    + "A smaller heap starts faster.";
        } else {
            pressure = "ok";
            advice = "Heap " + heapMib + " MB is right for this mod set.";
        }

        String mods = installedModsCsv == null ? "" : installedModsCsv.toLowerCase(Locale.ROOT);
        if (mods.contains("sodium") && !mods.contains("indium")) {
            advice += " Sodium is installed without Indium: Fabric mods that draw will not render.";
        }
        if (!mods.contains("sodium") && !mods.contains("embeddium")) {
            advice += " No Sodium: the single biggest frame-rate win is missing.";
        }

        return "{\"advice\":\"" + escape(advice) + "\",\"heap_mib\":" + heapMib
                + ",\"pressure\":\"" + pressure + "\"}";
    }

    private static String escape(String text) {
        return text.replace("\\", "\\\\").replace("\"", "\\\"");
    }
}
