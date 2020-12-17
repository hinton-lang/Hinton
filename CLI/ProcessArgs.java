package org.hinton_lang.CLI;

import java.io.IOException;
import java.nio.file.Path;

import org.hinton_lang.Hinton;

public class ProcessArgs {
    String[] args;

    public ProcessArgs(String... args) {
        this.args = args;
    }

    public void run() throws IOException {
        Path base = Path.of(System.getProperty("user.dir"));

        if (args.length > 1) {
            System.out.println("Usage: ht [script]");
            System.exit(64);
        } else if (args.length == 1) {
            // Resolve the given path
            Path file = Path.of(args[0]);
            String filePath = base.resolve(file).normalize().toString();

            // Run the file
            Hinton.runFile(filePath);
        } else {
            Hinton.runPrompt();
        }
    }
}
