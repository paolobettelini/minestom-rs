package org.example;

import java.lang.reflect.Field;
import java.lang.reflect.Modifier;

import net.minestom.server.entity.EntityType;
import net.minestom.server.particle.Particle;

public class Main {

    static {
        System.out.println("Loading ConsumerCallback class");
        try {
            String libraryPath = System.getProperty("java.library.path");
            System.out.println("Library path: " + libraryPath);
            System.loadLibrary("minestom");
            System.out.println("Loaded native library successfully");
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