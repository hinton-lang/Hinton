package org.hinton_lang.Interpreter.HintonArrays;

import java.util.ArrayList;
import java.util.List;

import org.hinton_lang.Interpreter.HintonFunctions.HintonCallable;

/**
 * Method for adding an element to the end of an array.
 */
public class ArrayPush implements HintonCallable {
    ArrayList<Object> arr;

    public ArrayPush(ArrayList<Object> arr) {
        this.arr = arr;
    }

    @Override
    public Object call(List<Object> arguments) {
        this.arr.add(arguments.get(0));
        return this.arr.size();
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
        return "<Function: [Array].push>";
    }
}
