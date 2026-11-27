package net.alpinoxi.bridge;

import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.UUID;

/**
 * The world one player opens to their friends.
 *
 * <p>Hosting is the feature that makes the friend list worth having: a player
 * opens their single-player world, the launcher publishes a short code, and a
 * friend who has that code is one click from joining. The rules live here so the
 * launcher and the game cannot disagree about what "open" means.
 *
 * <p>What this class does not do is the network. Opening a port, answering a
 * handshake and forwarding the connection belong to the transport; these are the
 * decisions around it, and they are the part that goes wrong quietly.
 */
public final class HostWorld {
    /** How open the world is. */
    public enum Access {
        /** Nobody. The default: a world is private until its owner says otherwise. */
        CLOSED("closed"),
        /** Everyone in the friend book, no code needed. */
        FRIENDS("friends"),
        /** Anyone with the code. */
        CODE("code");

        private final String id;

        Access(String id) {
            this.id = id;
        }

        public String id() {
            return id;
        }

        public static Access byId(String id) {
            for (Access access : values()) {
                if (access.id.equals(id)) {
                    return access;
                }
            }
            return CLOSED;
        }
    }

    /** One world that is open right now. */
    public record World(String id, String name, String ownerKey, Access access, int players,
                        int maxPlayers, boolean whitelistOnly) {
        public boolean full() {
            return players >= maxPlayers;
        }

        /** The shareable code, derived from the world id so it never changes. */
        public String code() {
            return codeFor(id);
        }
    }

    /** What happened when a player asked to join. */
    public enum JoinResult {
        OK,
        CLOSED,
        FULL,
        NOT_INVITED,
        NO_SUCH_WORLD,
        ALREADY_INSIDE;

        public String message() {
            return switch (this) {
                case OK -> "Joining…";
                case CLOSED -> "That world is not open.";
                case FULL -> "That world is full.";
                case NOT_INVITED -> "You need an invite for that world.";
                case NO_SUCH_WORLD -> "That world is no longer open.";
                case ALREADY_INSIDE -> "You are already in that world.";
            };
        }
    }

    private static final Map<String, World> OPEN = new LinkedHashMap<>();
    private static final Map<String, String> INVITES = new LinkedHashMap<>();
    private static String currentWorldId;
    private static int maxPlayers = 8;

    private HostWorld() {
    }

    public static int maxPlayers() {
        return maxPlayers;
    }

    /** A server cap is a promise about the host's connection, not a preference. */
    public static void setMaxPlayers(int players) {
        maxPlayers = Math.max(2, Math.min(16, players));
    }

    /**
     * A world code: four letters and four digits, from the world id's own bytes.
     *
     * <p>Derived rather than random so the same world always hands out the same
     * code, and a player who writes it down can use it next week.
     */
    public static String codeFor(String worldId) {
        if (worldId == null || worldId.isBlank()) {
            return "";
        }
        // The letters come from a set with no look-alikes: no O/0, no I/1.
        final String letters = "ABCDEFGHJKLMNPQRSTUVWXYZ";
        final UUID uuid = UUID.nameUUIDFromBytes(worldId.getBytes(java.nio.charset.StandardCharsets.UTF_8));
        long bits = uuid.getMostSignificantBits() ^ uuid.getLeastSignificantBits();
        StringBuilder code = new StringBuilder(8);
        for (int i = 0; i < 4; ++i) {
            code.append(letters.charAt((int) Math.floorMod(bits >> (i * 5), letters.length())));
        }
        for (int i = 4; i < 8; ++i) {
            code.append((char) ('0' + Math.floorMod(bits >> (i * 5), 10L)));
        }
        return code.toString();
    }

    /** Opens a world, or updates the one already open. */
    public static synchronized World open(String worldId, String name, String ownerKey,
                                          Access access) {
        World world = new World(worldId, name == null ? "World" : name, ownerKey,
                access == null ? Access.CLOSED : access, 1, maxPlayers, access == Access.CODE);
        OPEN.put(worldId, world);
        return world;
    }

    /** Closes a world and forgets its invites. */
    public static synchronized void close(String worldId) {
        OPEN.remove(worldId);
        INVITES.keySet().removeIf(key -> key.startsWith(worldId + "|"));
        if (worldId.equals(currentWorldId)) {
            currentWorldId = null;
        }
    }

    /** Hands one player the code for a world. Only the owner can invite. */
    public static synchronized boolean invite(String worldId, String ownerKey, String guestKey) {
        World world = OPEN.get(worldId);
        if (world == null || !world.ownerKey().equals(ownerKey) || guestKey == null) {
            return false;
        }
        INVITES.put(worldId + "|" + guestKey, world.code());
        return true;
    }

    public static synchronized World find(String worldIdOrCode) {
        if (worldIdOrCode == null) {
            return null;
        }
        World direct = OPEN.get(worldIdOrCode);
        if (direct != null) {
            return direct;
        }
        for (World world : OPEN.values()) {
            if (world.code().equalsIgnoreCase(worldIdOrCode)) {
                return world;
            }
        }
        return null;
    }

    /**
     * Whether one player may join one world, and why not when they may not.
     *
     * <p>The order matters: a closed world says closed even to a friend, and a
     * full world says full even to the owner, because neither is an invite
     * problem and telling a player the wrong one sends them asking for help
     * they do not need.
     */
    public static synchronized JoinResult mayJoin(String worldIdOrCode, String guestKey,
                                                  boolean isFriend, String code) {
        World world = find(worldIdOrCode);
        if (world == null) {
            return JoinResult.NO_SUCH_WORLD;
        }
        if (world.id().equals(currentWorldId)) {
            return JoinResult.ALREADY_INSIDE;
        }
        if (world.access() == Access.CLOSED) {
            return JoinResult.CLOSED;
        }
        if (world.full()) {
            return JoinResult.FULL;
        }
        if (world.access() == Access.FRIENDS) {
            return isFriend ? JoinResult.OK : JoinResult.NOT_INVITED;
        }
        // CODE: either the right code, or an invite the owner already sent.
        String invited = INVITES.get(world.id() + "|" + guestKey);
        if (code != null && world.code().equalsIgnoreCase(code.trim())) {
            return JoinResult.OK;
        }
        return invited != null ? JoinResult.OK : JoinResult.NOT_INVITED;
    }

    /** Notes that this player is inside a world, so a second join is refused. */
    public static synchronized void enter(String worldId) {
        currentWorldId = worldId;
        World world = OPEN.get(worldId);
        if (world != null) {
            OPEN.put(worldId, new World(world.id(), world.name(), world.ownerKey(), world.access(),
                    world.players() + 1, world.maxPlayers(), world.whitelistOnly()));
        }
    }

    public static synchronized void leave() {
        if (currentWorldId == null) {
            return;
        }
        World world = OPEN.get(currentWorldId);
        if (world != null) {
            OPEN.put(currentWorldId, new World(world.id(), world.name(), world.ownerKey(),
                    world.access(), Math.max(0, world.players() - 1), world.maxPlayers(),
                    world.whitelistOnly()));
        }
        currentWorldId = null;
    }

    public static synchronized String current() {
        return currentWorldId;
    }

    /** Every open world, for the list a player picks from. */
    public static synchronized List<World> openWorlds() {
        return new ArrayList<>(OPEN.values());
    }
}
