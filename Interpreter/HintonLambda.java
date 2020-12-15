package org.hinton_lang.Interpreter;

import java.util.List;

import org.hinton_lang.Parser.AST.Expr;
import org.hinton_lang.Envornment.Environment;

/**
 * Represents a lambda expression.
 */
public class HintonLambda implements HintonCallable {
    private final Expr.Lambda declaration;
    // For functions declared within the another function's body,
    // we use the parent's function as the enclosing (closure) scope.
    private final Environment closure;

    public HintonLambda(Expr.Lambda declaration, Environment closure) {
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
            environment.defineVar(declaration.params.get(i).lexeme, arguments.get(i));
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
        return "<LambdaExpression>";
    }
}
