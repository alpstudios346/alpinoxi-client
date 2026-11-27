package net.alpinoxi.hud;

import net.alpinoxi.bridge.Friends;
import net.alpinoxi.bridge.HostWorld;

/** One-line friend and world summary for the HUD. */
public final class FriendsPanel {
    private FriendsPanel() {
    }

    /** "friends 3", or empty when nobody is around. */
    public static String onlineLabel() {
        return Friends.onlineLabel();
    }

    /**
     * "2 joinable" when a friend's world can be joined right now, else empty.
     * Kept apart from the online count because it is the only part a player can
     * act on.
     */
    public static String joinableLabel() {
        int joinable = Friends.joinable().size();
        return joinable == 0 ? "" : joinable + " joinable";
    }

    /** "code ABCD1234" while this player hosts, else empty. */
    public static String hostLabel() {
        String current = HostWorld.current();
        if (current == null) {
            return "";
        }
        HostWorld.World world = HostWorld.find(current);
        return world == null ? "" : "code " + world.code();
    }
}
