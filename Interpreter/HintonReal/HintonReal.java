package org.hinton_lang.Interpreter.HintonReal;

import java.util.HashMap;

import org.hinton_lang.Errors.RuntimeError;
import org.hinton_lang.Interpreter.NativeType;
import org.hinton_lang.Tokens.Token;

/**
 * Represents a real (double) in the Hinton language.
 */
public class HintonReal implements NativeType {
    private final double realValue;
    private final HashMap<String, Object> methods = new HashMap<>();

    /**
     * Wraps a Java Double so that it can be read as a Hinton Real by the
     * interpreter.
     * 
     * @param real The Java Double to be wrapped.
     */
    public HintonReal(double real) {
        this.realValue = real;

        // The methods for Hinton Real objects in the Hinton language
    }

    /**
     * Returns the raw double in this wrapper.
     * 
     * @return The raw double.
     */
    public Double getRaw() {
        return this.realValue;
    }

    /**
     * Return the Hinton type name for the object.
     */
    public String typeName() {
        return "Real";
    }

    /**
     * Gets the given Hinton Real property.
     * 
     * @param prop The property to be accessed.
     * @return The property's value.
     */
    public Object getProperty(Token prop) {
        if (methods.containsKey(prop.lexeme)) {
            return methods.get(prop.lexeme);
        }

        throw new RuntimeError(prop, "Property '" + prop.lexeme + "' does not exist on type 'Real'.");
    }

    /**
     * String representation of a Hinton Real.
     */
    @Override
    public String toString() {
        return String.valueOf(this.realValue);
    }

    /**
     * Formatted string representation of a Hinton Real.
     */
    @Override
    public String formattedStr() {
        return "\u001b[34m" + this.toString() + "\u001b[0m";
    }
}
