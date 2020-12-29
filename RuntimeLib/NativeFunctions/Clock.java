package org.hinton_lang.RuntimeLib.NativeFunctions;

import java.util.HashMap;

import org.hinton_lang.Interpreter.HintonFunctions.HintonCallable;

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
            public Object call(HashMap<Object, Object> arguments) {
                return (double) System.currentTimeMillis() / 1000.0;
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
                return "<HintonFunction: clock>";
            }
        };
    }

}
