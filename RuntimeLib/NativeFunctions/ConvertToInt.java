package org.hinton_lang.RuntimeLib.NativeFunctions;

import java.util.List;

import org.hinton_lang.Errors.RuntimeError;
import org.hinton_lang.Interpreter.HintonFunctions.HintonCallable;
import org.hinton_lang.Interpreter.Interpreter;

public class ConvertToInt implements NativeFunc {
    /**
     * Specifies the name of the function.
     */
    @Override
    public String getFuncName() {
        return "int";
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
            public Integer call(Interpreter interpreter, List<Object> arguments) {
                Object toBeConverted = arguments.get(0);

                if (toBeConverted instanceof String) {
                    return Integer.parseInt((String) toBeConverted);
                }

                throw new RuntimeError("Cannot cast \"" + toBeConverted + "\" to integer");
            }

            @Override
            public String toString() {
                return "<HintonFunction: int>";
            }
        };
    }
}
