package net.alpinoxi.bridge;

import net.fabricmc.loader.api.FabricLoader;

/**
 * The one piece of state every Alpinoxi mod shares in the game.
 *
 * <p>It is plain Java on purpose. The launcher writes the same rules natively,
 * but the game must keep working with no native library at all: a missing DLL
 * turns the HUD into defaults, never into a crash.
 *
 * <p>Every field here is read and written from the client thread only, except
 * {@link #chat}, which the network thread appends to.
 */
public final class HudState {
    /** Where an element is pinned. */
    public enum Anchor {
        TOP_LEFT("top_left", 4, 4),
        TOP_CENTER("top_center", 0, 4),
        TOP_RIGHT("top_right", -4, 4),
        BOTTOM_LEFT("bottom_left", 4, -4),
        BOTTOM_CENTER("bottom_center", 0, -4),
        BOTTOM_RIGHT("bottom_right", -4, -4);

        private final String id;
        private final int dx;
        private final int dy;

        Anchor(String id, int dx, int dy) {
            this.id = id;
            this.dx = dx;
            this.dy = dy;
        }

        public String id() {
            return id;
        }

        public int dx() {
            return dx;
        }

        public int dy() {
            return dy;
        }

        public static Anchor byId(String id) {
            for (Anchor anchor : values()) {
                if (anchor.id.equals(id)) {
                    return anchor;
                }
            }
            return null;
        }
    }

    /** One HUD element: what it is called, where it sits and whether it shows. */
    public record Element(String id, String label, Anchor anchor, boolean visible, int offsetX,
                          int offsetY) {
    }

    private static final Element[] ELEMENTS = {
            new Element("fps", "FPS", Anchor.TOP_LEFT, true, 0, 0),
            new Element("coords", "Coordinates", Anchor.TOP_LEFT, true, 0, 22),
            new Element("ping", "Ping", Anchor.TOP_LEFT, true, 0, 44),
            new Element("memory", "Memory", Anchor.TOP_RIGHT, true, 0, 0),
            new Element("clock", "Clock", Anchor.TOP_RIGHT, true, 0, 22),
            new Element("keystrokes", "Keystrokes", Anchor.BOTTOM_LEFT, true, 0, 0),
    };

    private static boolean hudEnabled = true;
    private static final java.util.Map<String, Element> OVERRIDES = new java.util.HashMap<>();

    private HudState() {
    }

    public static boolean hudEnabled() {
        return hudEnabled;
    }

    public static void setHudEnabled(boolean enabled) {
        hudEnabled = enabled;
    }

    /** The elements as they are right now, overrides applied. */
    public static java.util.List<Element> elements() {
        java.util.List<Element> list = new java.util.ArrayList<>(ELEMENTS.length);
        for (Element element : ELEMENTS) {
            list.add(OVERRIDES.getOrDefault(element.id(), element));
        }
        return list;
    }

    public static Element find(String id) {
        Element override = OVERRIDES.get(id);
        if (override != null) {
            return override;
        }
        for (Element element : ELEMENTS) {
            if (element.id().equals(id)) {
                return element;
            }
        }
        return null;
    }

    /** Moves one element. Returns false for an id or anchor that does not exist. */
    public static boolean setAnchor(String id, String anchorId) {
        Element element = find(id);
        Anchor anchor = Anchor.byId(anchorId);
        if (element == null || anchor == null) {
            return false;
        }
        OVERRIDES.put(id, new Element(element.id(), element.label(), anchor, element.visible(),
                element.offsetX(), element.offsetY()));
        return true;
    }

    /** Shows or hides one element. -1 for an unknown id, else 1 or 0. */
    public static int toggle(String id) {
        Element element = find(id);
        if (element == null) {
            return -1;
        }
        boolean visible = !element.visible();
        OVERRIDES.put(id, new Element(element.id(), element.label(), element.anchor(), visible,
                element.offsetX(), element.offsetY()));
        return visible ? 1 : 0;
    }

    // ---------- settings the launcher and the game must agree on ----------

    /** Written by the launcher, read here. Kept in memory: the file is the truth. */
    public static final class Settings {
        private static boolean fromLauncher;
        private static int heapMib = 4096;
        private static String instance = "1.21.1";

        private Settings() {
        }

        public static void apply(boolean present, int heap, String instanceName) {
            fromLauncher = present;
            if (heap > 0) {
                heapMib = heap;
            }
            if (instanceName != null && !instanceName.isBlank()) {
                instance = instanceName;
            }
        }

        public static boolean present() {
            return fromLauncher;
        }

        public static int heapMib() {
            return heapMib;
        }

        public static String instance() {
            return instance;
        }
    }

    // ---------- chat ----------

    /** One line of chat, from any provider. */
    public record ChatLine(String from, String provider, String channel, String text,
                           long timestamp, boolean mentionsYou) {
    }

    private static final java.util.Deque<ChatLine> CHAT = new java.util.ArrayDeque<>();
    private static final int CHAT_LIMIT = 200;
    private static int unread;
    private static String selfName = "";

    public static void setSelfName(String name) {
        selfName = name == null ? "" : name;
    }

    /** Feeds one line in. Returns true when the line mentioned the player. */
    public static boolean receive(String from, String provider, String channel, String text,
                                  long timestamp) {
        boolean mentioned = !selfName.isEmpty() && text.toLowerCase(java.util.Locale.ROOT)
                .contains(selfName.toLowerCase(java.util.Locale.ROOT));
        synchronized (CHAT) {
            CHAT.addLast(new ChatLine(from, provider, channel, text, timestamp, mentioned));
            while (CHAT.size() > CHAT_LIMIT) {
                CHAT.removeFirst();
            }
            if (mentioned) {
                unread++;
            }
        }
        return mentioned;
    }

    public static java.util.List<ChatLine> chat() {
        synchronized (CHAT) {
            return new java.util.ArrayList<>(CHAT);
        }
    }

    public static int unread() {
        synchronized (CHAT) {
            return unread;
        }
    }

    public static void clearUnread() {
        synchronized (CHAT) {
            unread = 0;
        }
    }

    // ---------- what the launcher could not tell us ----------

    /** True when the launcher put a bridge settings file beside the game. */
    public static boolean launcherPresent() {
        try {
            return FabricLoader.getInstance().getConfigDir().resolve("alpinoxi-bridge.json")
                    .toFile().isFile();
        } catch (RuntimeException error) {
            return false;
        }
    }
}
