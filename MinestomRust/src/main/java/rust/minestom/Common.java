package rust.minestom;

import net.minestom.server.MinecraftServer;
import net.minestom.server.coordinate.ChunkRange;
import net.minestom.server.event.GlobalEventHandler;
import net.minestom.server.event.player.AsyncPlayerConfigurationEvent;
import net.minestom.server.instance.Chunk;
import net.minestom.server.instance.InstanceContainer;
import net.minestom.server.instance.LightingChunk;
import net.minestom.server.utils.chunk.ChunkUtils;

import java.io.File;
import java.util.ArrayList;
import java.util.concurrent.CompletableFuture;

/**
 * Common utility functions for Minestom
 */
public class Common {
    /**
     * Loads an Anvil world into the given instance.
     * Called from Rust to load a world.
     *
     * @param instance The instance to load the world into
     * @param path     Path to the Anvil world directory
     */
    public static void loadAnvil(InstanceContainer instance, String path) {
        System.out.println("Loading world from Anvil... please wait");

        instance.setChunkSupplier(LightingChunk::new);
        instance.setChunkLoader(new net.minestom.server.instance.anvil.AnvilLoader(path));
        instance.setTimeRate(0);
        var chunks = new ArrayList<CompletableFuture<Chunk>>();
        ChunkRange.chunksInRange(0, 0, 32, (x, z) -> chunks.add(instance.loadChunk(x, z)));
        CompletableFuture.runAsync(() -> {
            System.out.println("Loading world lightning... please wait");
            CompletableFuture.allOf(chunks.toArray(CompletableFuture[]::new)).join();
            LightingChunk.relight(instance, instance.getChunks());
            System.out.println("All done!");
        });

        System.out.println("World loading completed successfully!");
    }
} 