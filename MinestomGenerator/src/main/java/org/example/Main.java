package org.example;


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
    }

    public static native void startServer();
    
    public static void main(String[] args) {
        startServer();
    }

} 