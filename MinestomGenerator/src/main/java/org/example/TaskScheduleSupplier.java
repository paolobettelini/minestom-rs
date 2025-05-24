package org.example;

import net.minestom.server.timer.TaskSchedule;
import java.util.function.Supplier;

public class TaskScheduleSupplier implements Supplier<TaskSchedule> {
    private final Runnable runnable;
    private boolean firstRun = true;
    private long delay = 0;
    private boolean shouldRepeat = false;
    private long interval = 0;

    static {
        System.out.println("TaskScheduleSupplier loaded!");
    }

    public TaskScheduleSupplier(Runnable runnable) {
        System.out.println("TaskScheduleSupplier constructor called");
        if (runnable == null) {
            throw new IllegalArgumentException("Runnable is null");
        }
        this.runnable = runnable;
    }

    @Override
    public TaskSchedule get() {
        System.err.println("[DEBUG-SUPPLIER] TaskScheduleSupplier.get() called");
        System.err.println("[DEBUG-SUPPLIER] Current state - firstRun: " + firstRun + ", delay: " + delay + ", shouldRepeat: " + shouldRepeat + ", interval: " + interval);
        
        try {
            // Run the task
            System.err.println("[DEBUG-SUPPLIER] About to run task");
            runnable.run();
            System.err.println("[DEBUG-SUPPLIER] Task completed successfully");

            // Handle scheduling
            if (firstRun && delay > 0) {
                firstRun = false;
                System.err.println("[DEBUG-SUPPLIER] First run with delay: " + delay + " ticks");
                return TaskSchedule.tick((int)delay);
            }

            if (shouldRepeat) {
                System.err.println("[DEBUG-SUPPLIER] Task repeating with interval: " + interval + " ticks");
                return TaskSchedule.tick((int)interval);
            }

            System.err.println("[DEBUG-SUPPLIER] Task stopping (no repeat)");
            return TaskSchedule.stop();

        } catch (Exception e) {
            System.err.println("[DEBUG-SUPPLIER] Error in task execution:");
            e.printStackTrace(System.err);
            System.err.println("[DEBUG-SUPPLIER] Stack trace:");
            for (StackTraceElement element : e.getStackTrace()) {
                System.err.println("    at " + element);
            }
            if (e.getCause() != null) {
                System.err.println("[DEBUG-SUPPLIER] Caused by:");
                e.getCause().printStackTrace(System.err);
            }
            return TaskSchedule.stop();
        }
    }

    public void setDelay(long delay) {
        System.err.println("[DEBUG-SUPPLIER] Setting delay to " + delay + " ticks");
        this.delay = delay;
    }

    public void setRepeat(long interval) {
        System.err.println("[DEBUG-SUPPLIER] Setting repeat with interval " + interval + " ticks");
        this.shouldRepeat = true;
        this.interval = interval;
    }
} 