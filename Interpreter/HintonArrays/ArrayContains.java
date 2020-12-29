package org.hinton_lang.Interpreter.HintonArrays;

import java.util.ArrayList;
import java.util.HashMap;

import org.hinton_lang.Interpreter.HintonFunctions.HintonCallable;

/**
 * Method for checking if an element is inside an array.
 */
public class ArrayContains implements HintonCallable {
    ArrayList<Object> arr;

    public ArrayContains(ArrayList<Object> arr) {
        this.arr = arr;
    }

    @Override
    public Object call(HashMap<Object, Object> arguments) {
        return this.arr.contains(arguments.get(0));
    }

    @Override
    public int minArity() {
        return 1;
    }

    @Override
    public int maxArity() {
        return 1;
    }

    @Override
    public String toString() {
        return "<Function: [Array].constains>";
    }
}