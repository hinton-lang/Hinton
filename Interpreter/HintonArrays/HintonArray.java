package org.hinton_lang.Interpreter.HintonArrays;

import java.util.ArrayList;
import java.util.HashMap;

import org.hinton_lang.Errors.RuntimeError;
import org.hinton_lang.Interpreter.NativeType;
import org.hinton_lang.Tokens.Token;

/**
 * Represents a Hinton Array in the Hinton language.
 */
public class HintonArray implements NativeType {
    private final ArrayList<Object> arr;
    private final HashMap<String, Object> methods = new HashMap<>();

    /**
     * Wraps a Java ArrayList of objects so that it can be interpreted as a Hinton
     * Array.
     * 
     * @param array The Java ArrayList to be wrapped.
     */
    public HintonArray(ArrayList<Object> array) {
        this.arr = array;

        // The methods for array objects in the Hinton language
        methods.put("length", arr.size());
        methods.put("push", new ArrayPush(this.arr));
        methods.put("pop", new ArrayPop(this.arr));
        methods.put("contains", new ArrayContains(this.arr));
        methods.put("forEach", new ArrayForEach(this.arr));
    }

    /**
     * Gets the given Hinton Array property.
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
     * Returns the raw ArrayList in this wrapper.
     * 
     * @return The raw ArrayList.
     */
    public Object getRaw() {
        return this.arr;
    }

    /**
     * Return the Hinton type name for the object.
     */
    public String typeName() {
        return "Array";
    }

    /**
     * Gets the array item at the provided index.
     * 
     * @param index The index of the item.
     * @return The array item at the provided index.
     */
    public Object getItem(int index) {
        // Support for negative indexing
        if (index < 0)
            index = this.arr.size() + index;

        // If even after adjusting for negative index the provided
        // index is out of range, we throw an error.
        if (index < 0 || index > (this.arr.size() - 1)) {
            throw new RuntimeError("Array index out of range.");
        }

        return this.arr.get(index);
    }

    /**
     * String representation of a Hinton Array.
     */
    @Override
    public String toString() {
        StringBuilder str = new StringBuilder("[");

        for (int i = 0; i < this.arr.size(); i++) {
            Object item = this.arr.get(i);

            if (item instanceof NativeType) {
                str.append(((NativeType) item).formattedStr());
            } else {
                str.append(item);
            }

            // Adds a comma separator
            if (i < this.arr.size() - 1)
                str.append(", ");
        }

        str.append("]");

        return str.toString();
    }

    /**
     * Formatted string representation of a Hinton Integer.
     */
    @Override
    public String formattedStr() {
        return this.toString();
    }
}
