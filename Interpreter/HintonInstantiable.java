package org.hinton_lang.Interpreter;

import java.util.List;

/**
 * Represents a new class instance.
 */
public interface HintonInstantiable {
    /**
     * Instantiates and executes the class.
     * 
     * @param interpreter The class's enclosing environment.
     * @param arguments   The arguments for this class instance.
     * @return The object returned by the class.
     */
    public Object instantiate(Interpreter interpreter, List<Object> arguments);

    /**
     * The number of arguments that the particular class accepts.
     * 
     * @return The number of arguments accepted by the class instance.
     */
    public int arity();
}