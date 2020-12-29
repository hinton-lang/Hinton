package org.hinton_lang.Interpreter.HintonFunctions;

import java.util.HashMap;

/**
 * Represents a function call.
 */
public interface HintonCallable {
    /**
     * Calls and executes the function.
     * 
     * @param arguments The arguments for this function call.
     * @return The object returned by the function.
     */
    public Object call(HashMap<Object, Object> arguments);

    /**
     * The minimum number of arguments that the particular function accepts.
     * 
     * @return The minimum number of arguments accepted by the function.
     */
    public int minArity();

    /**
     * The maximum number of arguments that the particular function accepts.
     * 
     * @return The maximum number of arguments accepted by the function.
     */
    public int maxArity();
}
