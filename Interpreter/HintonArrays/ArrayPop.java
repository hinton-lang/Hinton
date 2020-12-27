package org.hinton_lang.Interpreter.HintonArrays;

import java.util.ArrayList;
import java.util.List;

import org.hinton_lang.Interpreter.HintonFunctions.HintonCallable;

/**
 * Method for removing the last element of an array.
 */
public class ArrayPop implements HintonCallable {
    ArrayList<Object> arr;

    public ArrayPop(ArrayList<Object> arr) {
        this.arr = arr;
    }

    @Override
    public Object call(List<Object> arguments) {
        Object popped = this.arr.get(this.arr.size() - 1);
        this.arr.remove(this.arr.size() - 1);
        return popped;
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
        return "<Function: [Array].pop>";
    }
}
