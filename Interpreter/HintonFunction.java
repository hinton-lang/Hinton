package org.hinton_lang.Interpreter;

import java.util.List;
import org.hinton_lang.Parser.AST.Stmt;
import org.hinton_lang.Envornment.DecType;
import org.hinton_lang.Envornment.Environment;

/**
 * Represents a function declaration.
 */
public class HintonFunction implements HintonCallable {
    private final Stmt.Function declaration;
    // For functions declared within the another function's body,
    // we use the parent's function as the enclosing (closure) scope.
    private final Environment closure;

    public HintonFunction(Stmt.Function declaration, Environment closure) {
        this.declaration = declaration;
        this.closure = closure;
    }

    /**
     * Creates a scope and executes the function on every function call.
     */
    @Override
    public Object call(Interpreter interpreter, List<Object> arguments) {
        Environment environment = new Environment(closure);

        for (int i = 0; i < declaration.params.size(); i++) {
            // We define the passed arguments as variables available within
            // the scope of this particular function.
            environment.define(declaration.params.get(i), arguments.get(i), DecType.VARIABLE);
        }

        try {
            interpreter.executeBlock(declaration.body, environment);
        } catch (Return returnValue) {
            return returnValue.value;
        }

        return null;
    }

    /**
     * The number of parameters declared for this function.
     */
    @Override
    public int arity() {
        return declaration.params.size();
    }

    /**
     * String representation.
     */
    @Override
    public String toString() {
        return "<Function " + declaration.name.lexeme + ">";
    }
}
