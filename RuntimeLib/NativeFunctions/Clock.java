package org.hinton_lang.RuntimeLib.NativeFunctions;

import java.util.List;

import org.hinton_lang.Interpreter.HintonCallable;
import org.hinton_lang.Interpreter.Interpreter;

/**
 * Native Hinton function for measuring the execution time between two
 * statements.
 */
public class Clock implements NativeFunc {

    /**
     * Specifies the name of the function.
     */
    @Override
    public String getFuncName() {
        return "clock";
    }

    /**
     * Specifies the function's body.
     */
    @Override
    public HintonCallable getFunc() {
        return new HintonCallable() {
            @Override
            public int arity() {
                return 0;
            }

            @Override
            public Object call(Interpreter interpreter, List<Object> arguments) {
                return (double) System.currentTimeMillis() / 1000.0;
            }

            @Override
            public String toString() {
                return "<HintonFunction: clock>";
            }
        };
    }

}
