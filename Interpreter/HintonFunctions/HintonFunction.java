package org.hinton_lang.Interpreter.HintonFunctions;

import org.hinton_lang.Parser.AST.Stmt;
import org.hinton_lang.Envornment.Environment;
import org.hinton_lang.Interpreter.Interpreter;

/**
 * Represents a function declaration.
 */
public class HintonFunction extends AbstractHintonFunction {
    private Stmt.Function declaration;

    public HintonFunction(Interpreter interpret, Stmt.Function declaration, Environment closure) {
        super(interpret, declaration.params, declaration.body, closure);
        this.declaration = declaration;
    }

    /**
     * String representation.
     */
    @Override
    public String toString() {
        return "<Function " + declaration.name.lexeme + ">";
    }
}
