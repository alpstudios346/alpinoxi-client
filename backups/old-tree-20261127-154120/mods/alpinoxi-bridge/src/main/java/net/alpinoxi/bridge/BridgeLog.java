package net.alpinoxi.bridge;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/** One logger for every Alpinoxi mod, so logs are easy to filter. */
public final class BridgeLog {
    private static final Logger LOGGER = LoggerFactory.getLogger("Alpinoxi");

    private BridgeLog() {
    }

    public static void info(String message, Object... args) {
        LOGGER.info(message, args);
    }

    public static void warn(String message, Object... args) {
        LOGGER.warn(message, args);
    }

    public static void error(String message, Object... args) {
        LOGGER.error(message, args);
    }
}
