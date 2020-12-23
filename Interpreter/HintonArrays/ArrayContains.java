package org.hinton_lang.Interpreter.HintonArrays;

import java.util.ArrayList;
import java.util.List;

import org.hinton_lang.Interpreter.HintonFunctions.HintonCallable;
import org.hinton_lang.Interpreter.Interpreter;

/**
 * Method for checking if an element is inside an array.
 */
public class ArrayContains implements HintonCallable {
    ArrayList<Object> arr;

    public ArrayContains(ArrayList<Object> arr) {
        this.arr = arr;
    }

    @Override
    public Object call(Interpreter interpreter, List<Object> arguments) {
        return this.arr.contains(arguments.get(0));
    }

    @Override
    public int arity() {
        return 1;
    }

    @Override
    public String toString() {
        return "<Function: [Array].constains>";
    }
}