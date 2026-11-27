package net.alpinoxi.bridge;

import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;

/**
 * Who your friends are, and whether they are reachable.
 *
 * <p>Identity is {@code provider:uuid}, the same pair the launcher uses, so a
 * cracked name and a Microsoft account can never collide. Presence is three
 * states, because "online" alone cannot tell a player who is in a menu from one
 * whose world you can join.
 *
 * <p>Nothing here touches the network: the launcher pushes presence in, and the
 * game reads it. A friend book that cannot be refreshed still lists the friends.
 */
public final class Friends {
    /** Where a friend is right now. */
    public enum Presence {
        OFFLINE("offline"),
        LAUNCHER("launcher"),
        IN_GAME("in_game");

        private final String id;

        Presence(String id) {
            this.id = id;
        }

        public String id() {
            return id;
        }

        public static Presence byId(String id) {
            for (Presence presence : values()) {
                if (presence.id.equals(id)) {
                    return presence;
                }
            }
            return OFFLINE;
        }
    }

    /** One friend, as the list shows them. */
    public record Friend(String provider, String uuid, String name, Presence presence,
                         String worldId) {
        /** "provider:uuid": the only identity that is safe to compare. */
        public String key() {
            return provider + ":" + uuid;
        }

        public boolean joinable() {
            return presence == Presence.IN_GAME && worldId != null && !worldId.isBlank();
        }
    }

    /** What came of asking someone to be a friend. */
    public enum RequestState {
        NONE,
        SENT,
        RECEIVED,
        FRIENDS,
        BLOCKED;

        public static RequestState byId(String id) {
            for (RequestState state : values()) {
                if (state.name().toLowerCase(Locale.ROOT).equals(id)) {
                    return state;
                }
            }
            return NONE;
        }

        public String id() {
            return name().toLowerCase(Locale.ROOT);
        }
    }

    private static final Map<String, Friend> BOOK = new LinkedHashMap<>();
    private static final Map<String, RequestState> REQUESTS = new LinkedHashMap<>();

    private Friends() {
    }

    /** Replaces the book with what the launcher just sent. */
    public static synchronized void replace(List<Friend> friends) {
        BOOK.clear();
        for (Friend friend : friends) {
            BOOK.put(friend.key(), friend);
        }
    }

    /** One friend's presence changed. An unknown friend is added, not dropped. */
    public static synchronized void setPresence(String provider, String uuid, String name,
                                                Presence presence, String worldId) {
        String key = provider + ":" + uuid;
        Friend known = BOOK.get(key);
        String displayName = name != null && !name.isBlank()
                ? name
                : (known == null ? uuid : known.name());
        BOOK.put(key, new Friend(provider, uuid, displayName, presence, worldId));
    }

    public static synchronized List<Friend> all() {
        return new ArrayList<>(BOOK.values());
    }

    /** Friends you can join this second: in game, in a world that is shared. */
    public static synchronized List<Friend> joinable() {
        List<Friend> list = new ArrayList<>();
        for (Friend friend : BOOK.values()) {
            if (friend.joinable()) {
                list.add(friend);
            }
        }
        return list;
    }

    public static synchronized int onlineCount() {
        int online = 0;
        for (Friend friend : BOOK.values()) {
            if (friend.presence() != Presence.OFFLINE) {
                online++;
            }
        }
        return online;
    }

    /** The one-line label the HUD shows: empty when nobody is around. */
    public static synchronized String onlineLabel() {
        int online = onlineCount();
        return online == 0 ? "" : "friends " + online;
    }

    // ---------- requests ----------

    public static synchronized RequestState stateOf(String provider, String uuid) {
        return REQUESTS.getOrDefault(provider + ":" + uuid, RequestState.NONE);
    }

    public static synchronized RequestState request(String provider, String uuid) {
        String key = provider + ":" + uuid;
        RequestState state = REQUESTS.getOrDefault(key, RequestState.NONE);
        if (state == RequestState.BLOCKED || state == RequestState.FRIENDS) {
            return state;
        }
        REQUESTS.put(key, RequestState.SENT);
        return RequestState.SENT;
    }

    public static synchronized RequestState accept(String provider, String uuid) {
        String key = provider + ":" + uuid;
        if (REQUESTS.getOrDefault(key, RequestState.NONE) == RequestState.BLOCKED) {
            return RequestState.BLOCKED;
        }
        REQUESTS.put(key, RequestState.FRIENDS);
        return RequestState.FRIENDS;
    }

    public static synchronized RequestState decline(String provider, String uuid) {
        REQUESTS.put(provider + ":" + uuid, RequestState.NONE);
        return RequestState.NONE;
    }

    /** Removes a friend: the book, the request state, and the label with them. */
    public static synchronized void remove(String provider, String uuid) {
        String key = provider + ":" + uuid;
        BOOK.remove(key);
        REQUESTS.remove(key);
    }

    /** Reads "provider:uuid". False for anything else, so a typo adds nobody. */
    public static boolean parseKey(String key, String[] out) {
        if (key == null) {
            return false;
        }
        int colon = key.indexOf(':');
        if (colon <= 0 || colon == key.length() - 1) {
            return false;
        }
        out[0] = key.substring(0, colon);
        out[1] = key.substring(colon + 1);
        return true;
    }
}
