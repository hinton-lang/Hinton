package org.hinton_lang.Interpreter.HintonArrays;

import java.util.ArrayList;
import java.util.HashMap;

import org.hinton_lang.Errors.RuntimeError;
import org.hinton_lang.Tokens.Token;

/**
 * Represents an array in the Hinton language.
 */
public class HintonArray {
    private ArrayList<Object> arr;
    private HashMap<String, Object> methods = new HashMap<>();

    /**
     * Wraps an ArrayList of objects so that it can be interpreted as a Hinton
     * array.
     * 
     * @param array The ArrayList to be wrapped.
     */
    public HintonArray(ArrayList<Object> array) {
        this.arr = array;

        // The methods for array objects in the Hinton language
        methods.put("length", arr.size());
        methods.put("push", new ArrayPush(this.arr));
        methods.put("pop", new ArrayPop(this.arr));
        methods.put("contains", new ArrayContains(this.arr));
    }

    /**
     * Gets the given array property.
     * 
     * @param prop The property to be accessed.
     * @return The property's value.
     */
    public Object getProperty(Token prop) {
        if (methods.containsKey(prop.lexeme)) {
            return methods.get(prop.lexeme);
        }

        throw new RuntimeError(prop, "Property '" + prop.lexeme + "' does not exist on type 'Array'.");
    }

    /**
     * String representation of a Hinton array.
     */
    @Override
    public String toString() {
        return arr.toString();
    }
}
