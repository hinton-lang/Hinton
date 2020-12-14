package org.hinton_lang;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.nio.charset.Charset;
import java.nio.file.Files;
import java.nio.file.Paths;
import java.util.List;

// Project-specific
import org.hinton_lang.Lexer.Lexer;
import org.hinton_lang.Tokens.*;
import org.hinton_lang.Parser.*;
import org.hinton_lang.Parser.AST.*;
import org.hinton_lang.Errors.RuntimeError;
import org.hinton_lang.Interpreter.Interpreter;

public class Hinton {
    private static final Interpreter interpreter = new Interpreter();
    static boolean hadError = false;
    static boolean hadRuntimeError = false;

    public static void main(String[] args) throws IOException {
        // runFile("/Users/faustotnc/Documents/GitHub/Hinton-Lang/src/org/hinton_lang/m.ht");

        if (args.length > 1) {
            System.out.println("Usage: ht [script]");
            System.exit(64);
        } else if (args.length == 1) {
            runFile(args[0]);
        } else {
            runPrompt();
        }
    }

    /**
     * Rus a file containing Hinton source code.
     * 
     * @param path The path to the file.
     * @throws IOException Error when the file is not found.
     */
    private static void runFile(String path) throws IOException {
        byte[] bytes = Files.readAllBytes(Paths.get(path));
        run(new String(bytes, Charset.defaultCharset()));

        // Indicate an error in the exit code.
        if (hadError)
            System.exit(65);
        if (hadRuntimeError)
            System.exit(70);
    }

    /**
     * Runs a REPL to execute Hinton code.
     * 
     * @throws IOException Error when the line cannot be read.
     */
    private static void runPrompt() throws IOException {
        InputStreamReader input = new InputStreamReader(System.in);
        BufferedReader reader = new BufferedReader(input);

        for (;;) {
            System.out.print(">> ");
            String line = reader.readLine();
            if (line == null)
                break;
            run(line);
            hadError = false;
        }
    }

    /**
     * Executes the source code (source file or REPL).
     * 
     * @param source The source code.
     */
    private static void run(String source) {
        Lexer lexer = new Lexer(source);
        List<Token> tokens = lexer.lexTokens();

        Parser parser = new Parser(tokens);
        List<Stmt> statements = parser.parse();

        // Stop if there was a syntax error.
        if (hadError)
            return;

        interpreter.interpret(statements);

        // // Prints the tokens - Debug only
        // for (Token token : tokens) {
        // System.out.println(token);
        // }
    }

    /**
     * Reports an error to the console
     * 
     * @param line    The line of source code that caused the error.
     * @param where   The position fo the error.
     * @param message An error message.
     */
    private static void report(int line, int column, String where, String message) {
        System.err.println("[line " + line + ":" + column + "] Error" + where + ": " + message);
        hadError = true;
    }

    /**
     * Reports a generic error.
     * 
     * @param line    The line of the error.
     * @param message The error message.
     */
    static void error(int line, int column, String message) {
        report(line, column, "", message);
    }

    /**
     * Reports a token error.
     * 
     * @param token   The token that is causing the error.
     * @param message The error message.
     */
    public static void error(Token token, String message) {
        if (token.type == TokenType.END_OF_FILE) {
            report(token.linePos, token.columnPos, " at end", message);
        } else {
            report(token.linePos, token.columnPos, " at '" + token.lexeme + "'", message);
        }
    }

    public static void runtimeError(RuntimeError error) {
        System.err.println(error.getMessage() + "\n[line " + error.token.linePos + "]");
        hadRuntimeError = true;
    }
}
