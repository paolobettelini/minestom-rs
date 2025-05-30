package rust.minestom;

import net.minestom.server.command.CommandSender;
import net.minestom.server.command.builder.condition.CommandCondition;

public class CommandConditionCallback implements CommandCondition {
    private final long callbackId;

    public CommandConditionCallback(long callbackId) {
        //System.out.println("Creating CommandConditionCallback with id: " + callbackId);
        this.callbackId = callbackId;
    }

    @Override
    public boolean canUse(CommandSender sender, String commandString) {
        //System.out.println("Checking command condition with callback id: " + callbackId);
        //System.out.println("Command string: " + commandString);
        return checkCondition(callbackId, sender);
    }

    // Native method that will be implemented in Rust
    private native boolean checkCondition(long callbackId, CommandSender sender);
} 