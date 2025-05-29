package rust.wsee;

import net.minestom.server.coordinate.Pos;
import net.minestom.server.instance.Instance;
import net.worldseed.multipart.GenericModelImpl;

import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;

/**
 * Generic callback wrapper for Rust implementations of GenericModelImpl.
 * Must be on the server classpath alongside the library.
 */
public class GenericModelCallback extends GenericModelImpl {
    private final long callbackId;

    /** Called from Rust to create a new model instance */
    public GenericModelCallback(long callbackId) {
        super();
        this.callbackId = callbackId;
    }

    private static native String nativeGetId(long callbackId);
    private static native void nativeInit(long callbackId,
                                          @Nullable Instance instance,
                                          @NotNull Pos position);

    @Override
    public String getId() {
        return nativeGetId(callbackId);
    }

    @Override
    public void init(@Nullable Instance instance,
                     @NotNull Pos position) {
        nativeInit(callbackId, instance, position);
        super.init(instance, position, 1.0f);
    }
}
