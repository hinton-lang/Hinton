package org.hinton_lang.Interpreter.HintonArrays;

import java.util.ArrayList;
import java.util.List;

import org.hinton_lang.Interpreter.HintonFunctions.HintonCallable;
import org.hinton_lang.Interpreter.Interpreter;

/**
 * Method for adding an element to the end of an array.
 */
public class ArrayPush implements HintonCallable {
    ArrayList<Object> arr;

    public ArrayPush(ArrayList<Object> arr) {
        this.arr = arr;
    }

    @Override
    public Object call(Interpreter interpreter, List<Object> arguments) {
        this.arr.add(arguments.get(0));
        return this.arr.size();
    }

    @Override
    public int arity() {
        return 1;
    }

    @Override
    public String toString() {
        return "<Function: [Array].push>";
    }
}
