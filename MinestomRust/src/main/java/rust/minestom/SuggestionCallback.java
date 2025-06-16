package rust.minestom;

import net.minestom.server.command.CommandSender;
import net.minestom.server.command.builder.CommandContext;
import net.minestom.server.command.builder.suggestion.Suggestion;

public class SuggestionCallback implements net.minestom.server.command.builder.suggestion.SuggestionCallback {
    private final long callbackId;

    public SuggestionCallback(long callbackId) {
        this.callbackId = callbackId;
    }

    @Override
    public void apply(CommandSender sender, CommandContext context, Suggestion suggestion) {
        applySuggestion(callbackId, sender, context, suggestion);
    }

    // Native method that will be implemented in Rust
    private native void applySuggestion(long callbackId, CommandSender sender, CommandContext context, Suggestion suggestion);
}