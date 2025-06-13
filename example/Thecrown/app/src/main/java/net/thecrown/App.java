package net.thecrown;

import net.minestom.server.entity.EntityType;
import net.minestom.server.particle.Particle;

public class App {

    static {
        try {
            String libName = System.getProperty("lib.name", "minestom");
            String libraryPath = System.getProperty("java.library.path");
            System.out.println("Library path: " + libraryPath);
            System.loadLibrary(libName);
        } catch (UnsatisfiedLinkError e) {
            System.err.println("Failed to load native library: " + e.getMessage());
            e.printStackTrace();
        }
        
        // INIT EntityType
        doNothing(EntityType.ARMOR_STAND);
        doNothing(Particle.HEART);
        doNothing(Particle.NOTE);
    }

    static <T> void doNothing(T t) {}

    public static native void startServer();

    public static void main(String[] args) {
        startServer();
    }
}
