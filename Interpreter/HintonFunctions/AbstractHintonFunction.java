package org.hinton_lang.Interpreter.HintonFunctions;

import java.util.List;

import org.hinton_lang.Interpreter.Interpreter;
import org.hinton_lang.Interpreter.ControlStmts.Return;
import org.hinton_lang.Interpreter.HintonNull.HintonNull;
import org.hinton_lang.Parser.AST.Stmt;
import org.hinton_lang.Envornment.DecType;
import org.hinton_lang.Envornment.Environment;

public abstract class AbstractHintonFunction implements HintonCallable {
    public final List<Stmt.Parameter> parameters;
    public final List<Stmt> body;
    public final Interpreter interpreter;
    // For functions declared within the another function's body,
    // we use the parent's function as the enclosing (closure) scope.
    private final Environment closure;
    // The scope for this functions
    private final Environment environment;

    private int minArity, maxArity = 0;

    public AbstractHintonFunction(Interpreter interpreter, List<Stmt.Parameter> params, List<Stmt> body,
            Environment closure) {
        this.parameters = params;
        this.body = body;
        this.closure = closure;
        this.environment = new Environment(this.closure);
        this.interpreter = interpreter;

        // Default value for parameters.
        for (int i = 0; i < params.size(); i++) {
            Stmt.Parameter param = params.get(i);
            // We define the passed arguments as variables available within
            // the scope of this particular function.
            environment.define(param.name, interpreter.evaluate(param.defVal), DecType.VARIABLE);

            // Records the minimum and maximum number of
            // parameters accepted by this function.
            if (!param.isOptnl) {
                minArity = minArity + 1;
            }
            maxArity = maxArity + 1;
        }
    }

    /**
     * Creates a scope and executes the function on every function call.
     */
    @Override
    public Object call(List<Object> arguments) {

        // Assigns the given arguments to the parameters
        for (int i = 0; i < arguments.size(); i++) {
            // We define the passed arguments as variables available within
            // the scope of this particular function.
            environment.assign(this.parameters.get(i).name, arguments.get(i));
        }

        // Executes the function
        try {
            interpreter.executeBlock(this.body, this.environment);
        } catch (Return returnValue) {
            return returnValue.value;
        }

        // Returns null on void block
        return new HintonNull();
    }

    /**
     * The number of required parameters declared for this function.
     */
    @Override
    public int minArity() {
        return this.minArity;
    }

    /**
     * The number of optional parameters declared for this function.
     */
    @Override
    public int maxArity() {
        return this.maxArity;
    }
}
