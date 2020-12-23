package org.hinton_lang.RuntimeLib.NativeFunctions;

import java.util.List;

import org.hinton_lang.Helper.Helper;
import org.hinton_lang.Interpreter.HintonFunctions.HintonCallable;
import org.hinton_lang.Interpreter.Interpreter;

/**
 * Native Hinton function for printing output to the command prompt.
 */
public class Print implements NativeFunc {

    /**
     * Specifies the name of the function.
     */
    @Override
    public String getFuncName() {
        return "print";
    }

    /**
     * Specifies the function's body.
     */
    @Override
    public HintonCallable getFunc() {
        return new HintonCallable() {
            @Override
            public int arity() {
                return 1;
            }

            @Override
            public Void call(Interpreter interpreter, List<Object> arguments) {
                System.out.println(Helper.stringify(arguments.get(0)));
                return null;
            }

            @Override
            public String toString() {
                return "<HintonFunction: print>";
            }
        };
    }
}
