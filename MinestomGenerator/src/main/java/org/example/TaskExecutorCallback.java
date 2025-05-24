package org.example;

public class TaskExecutorCallback implements Runnable {
    private final long callbackId;

    public TaskExecutorCallback(long callbackId) {
        System.out.println("Creating TaskExecutorCallback with id: " + callbackId);
        this.callbackId = callbackId;
    }

    @Override
    public void run() {
        System.out.println("Executing task with callback id: " + callbackId);
        executeTask(callbackId);
    }

    // Native method that will be implemented in Rust
    private native void executeTask(long callbackId);
} 