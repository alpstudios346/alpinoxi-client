package net.alpinoxi.perf;

import java.util.Arrays;

/**
 * Client-side frame-time meter.
 *
 * <p>Plain and honest: it keeps the last 600 frame times and reports percentiles
 * from a sorted copy. Average FPS alone hides stutter, so the 1% low is what the
 * HUD shows next to it.
 */
public final class FrameMeter {
    private static final int CAPACITY = 600;

    private static final float[] FRAMES = new float[CAPACITY];
    private static final float[] SCRATCH = new float[CAPACITY];
    private static int cursor;
    private static int count;

    private FrameMeter() {
    }

    public static void record(float frameMillis) {
        if (!(frameMillis > 0.0f) || Float.isInfinite(frameMillis)) {
            return;
        }
        FRAMES[cursor] = frameMillis;
        cursor = (cursor + 1) % CAPACITY;
        count = Math.min(count + 1, CAPACITY);
    }

    public static int samples() {
        return count;
    }

    public static float averageFps() {
        if (count == 0) {
            return 0.0f;
        }
        float total = 0.0f;
        for (int index = 0; index < count; index++) {
            total += FRAMES[index];
        }
        float mean = total / count;
        return mean > 0.0f ? 1_000.0f / mean : 0.0f;
    }

    /** The FPS of the slowest 1% of frames. */
    public static float onePercentLowFps() {
        float worst = percentileMillis(99.0f);
        return worst > 0.0f ? 1_000.0f / worst : 0.0f;
    }

    public static float percentileMillis(float percentile) {
        if (count == 0) {
            return 0.0f;
        }
        System.arraycopy(FRAMES, 0, SCRATCH, 0, count);
        Arrays.sort(SCRATCH, 0, count);
        int rank = Math.round((percentile / 100.0f) * (count - 1));
        return SCRATCH[Math.min(Math.max(rank, 0), count - 1)];
    }

    public static void reset() {
        cursor = 0;
        count = 0;
    }
}
