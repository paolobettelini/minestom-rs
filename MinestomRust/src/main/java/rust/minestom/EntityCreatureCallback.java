package rust.minestom;

import net.minestom.server.coordinate.Point;
import net.minestom.server.entity.EntityCreature;
import net.minestom.server.entity.EntityType;
import net.minestom.server.entity.Player;
import net.minestom.server.entity.damage.DamageType;
import net.minestom.server.entity.damage.EntityDamage;
import net.minestom.server.instance.Instance;
import net.minestom.server.coordinate.Pos;
import net.minestom.server.registry.DynamicRegistry;
import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;

/**
 * Generic callback wrapper for Rust implementations of EntityCreature.
 * Must be on the server classpath alongside the library.
 */
public class EntityCreatureCallback extends EntityCreature {
    private final long callbackId;

    /** Called from Rust to create a new EntityCreature instance */
    public EntityCreatureCallback(long callbackId, @NotNull EntityType type) {
        super(type);
        this.callbackId = callbackId;
    }

    private static native void nativeUpdateNewViewer(long callbackId, @NotNull Player player);
    private static native void nativeUpdateOldViewer(long callbackId, @NotNull Player player);
    private static native void nativeTick(long callbackId, long time);
    private static native boolean nativeDamage(long callbackId,
                                               @NotNull DynamicRegistry.Key<DamageType> type,
                                               float amount);
    private static native void nativeRemove(long callbackId);

    @Override
    public void updateNewViewer(@NotNull Player player) {
        // First let Rust handle any custom behavior, then call super
        nativeUpdateNewViewer(callbackId, player);
        super.updateNewViewer(player);
    }

    @Override
    public void updateOldViewer(@NotNull Player player) {
        nativeUpdateOldViewer(callbackId, player);
        super.updateOldViewer(player);
    }

    @Override
    public void tick(long time) {
        nativeTick(callbackId, time);
        super.tick(time);
    }

    /*@Override // TODO
    public boolean damage(@NotNull DynamicRegistry.Key<DamageType> type, float amount) {
        // Let Rust decide if it wants to cancel or augment damage
        boolean rustResult = nativeDamage(callbackId, type, amount);
        // Always pass through to the regular damage logic (unless Rust returned false to indicate “cancel”)
        if (!rustResult) {
            return false;
        }
        return super.damage(type, amount);
    }*/

    @Override
    public void remove() {
        // Let Rust do any custom cleanup or animations, then remove
        nativeRemove(callbackId);
        super.remove();
    }
}
