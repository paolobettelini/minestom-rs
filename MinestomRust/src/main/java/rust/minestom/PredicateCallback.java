package rust.minestom;

import net.minestom.server.entity.Player;
import java.util.function.Predicate;

public class PredicateCallback implements Predicate<Player> {
    private final long callbackId;

    public PredicateCallback(long callbackId) {
        this.callbackId = callbackId;
    }

    @Override
    public boolean test(Player player) {
        return testPlayer(callbackId, player);
    }

    private native boolean testPlayer(long callbackId, Player player);
} 