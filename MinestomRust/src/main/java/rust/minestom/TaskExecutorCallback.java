package rust.minestom;

import java.io.FileWriter;
import java.io.IOException;
import java.io.PrintWriter;
import java.util.Map;
import java.io.File;

public class TaskExecutorCallback implements Runnable {
    private final long callbackId;
    private static volatile boolean initialized = false;

    static {
        System.out.println("Loading TaskExecutorCallback class");
        try {
            System.loadLibrary("minestom");
            initialized = true;
            System.out.println("Loaded native library successfully");
        } catch (UnsatisfiedLinkError e) {
            System.err.println("Failed to load native library: " + e.getMessage());
            e.printStackTrace();
        }
    }

    public TaskExecutorCallback(long callbackId) {
        System.out.println("Creating TaskExecutorCallback with id: " + callbackId);
        this.callbackId = callbackId;
    }

    @Override
    public void run() {
        System.err.println("[JAVA-TASK] Starting task execution with ID: " + callbackId);
        try {
            System.err.println("[JAVA-TASK] About to call native executeTask");
            executeTask(callbackId);
            System.err.println("[JAVA-TASK] Native executeTask completed successfully");
        } catch (Throwable t) {
            System.err.println("[JAVA-TASK] ERROR: Task execution failed!");
            t.printStackTrace(System.err);
            
            // Print the current thread's state
            Thread currentThread = Thread.currentThread();
            System.err.println("[JAVA-TASK] Thread state: " + currentThread.getState());
            System.err.println("[JAVA-TASK] Thread name: " + currentThread.getName());
            System.err.println("[JAVA-TASK] Is thread interrupted: " + currentThread.isInterrupted());
            
            // Print the full stack trace
            StackTraceElement[] stackTrace = t.getStackTrace();
            System.err.println("[JAVA-TASK] Full stack trace:");
            for (StackTraceElement element : stackTrace) {
                System.err.println("    at " + element);
            }
        }
    }

    // Native method that will be implemented in Rust
    private native void executeTask(long callbackId);
} 