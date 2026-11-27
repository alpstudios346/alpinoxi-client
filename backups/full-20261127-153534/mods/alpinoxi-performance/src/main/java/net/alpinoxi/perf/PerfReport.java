package net.alpinoxi.perf;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import net.alpinoxi.bridge.Native;

/**
 * Asks the Rust core what the numbers mean.
 *
 * <p>The mod measures; Rust diagnoses. If the bridge is missing, the line says so
 * instead of guessing, which keeps the recommendation trustworthy.
 */
public final class PerfReport {
    private static String last = "Measuring…";

    private PerfReport() {
    }

    public static void refresh() {
        if (!Native.ensureLoaded()) {
            last = "Alpinoxi bridge unavailable";
            return;
        }
        try {
            long maxHeap = Runtime.getRuntime().maxMemory() / (1024L * 1024L);
            long usedHeap = (Runtime.getRuntime().totalMemory() - Runtime.getRuntime().freeMemory())
                    / (1024L * 1024L);
            JsonObject report = JsonParser.parseString(
                            Native.performanceReport((int) maxHeap, InstalledMods.csv()))
                    .getAsJsonObject();
            String advice = report.get("advice").getAsString();
            last = String.format("%s · heap %d/%d MB", advice, usedHeap, maxHeap);
        } catch (RuntimeException error) {
            last = "Report unavailable";
        }
    }

    public static String diagnosisLine() {
        return last;
    }
}
