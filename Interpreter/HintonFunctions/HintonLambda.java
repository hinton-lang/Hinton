package org.hinton_lang.Interpreter.HintonFunctions;

import org.hinton_lang.Parser.AST.Expr;
import org.hinton_lang.Envornment.Environment;
import org.hinton_lang.Interpreter.Interpreter;

/**
 * Represents a lambda expression.
 */
public class HintonLambda extends AbstractHintonFunction {
    public HintonLambda(Interpreter interpret, Expr.Lambda declaration, Environment closure) {
        super(interpret, declaration.params, declaration.body, closure);
    }

    /**
     * String representation.
     */
    @Override
    public String toString() {
        return "<LambdaExpression>";
    }
}
