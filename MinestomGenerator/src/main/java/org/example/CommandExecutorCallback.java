package org.example;

import net.minestom.server.command.CommandSender;
import net.minestom.server.command.builder.CommandContext;
import net.minestom.server.command.builder.CommandExecutor;

public class CommandExecutorCallback implements CommandExecutor {
    private final long callbackId;

    public CommandExecutorCallback(long callbackId) {
        //System.out.println("Constructor called with: " + callbackId);
        this.callbackId = callbackId;
    }

    @Override
    public void apply(CommandSender sender, CommandContext context) {
        System.out.println("Executing command with callback id: " + callbackId);
        executeCommand(callbackId, sender, context);
    }

    // Native method that will be implemented in Rust
    private native void executeCommand(long callbackId, CommandSender sender, CommandContext context);
} 