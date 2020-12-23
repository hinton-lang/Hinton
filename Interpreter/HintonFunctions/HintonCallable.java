package org.hinton_lang.Interpreter.HintonFunctions;

import org.hinton_lang.Interpreter.Interpreter;

import java.util.List;

/**
 * Represents a function call.
 */
public interface HintonCallable {
    /**
     * Calls and executes the function.
     * 
     * @param interpreter The function's enclosing environment.
     * @param arguments   The arguments for this function call.
     * @return The object returned by the function.
     */
    public Object call(Interpreter interpreter, List<Object> arguments);

    /**
     * The number of arguments that the particular function accepts.
     * 
     * @return The number of arguments accepted by the function.
     */
    public int arity();
}
