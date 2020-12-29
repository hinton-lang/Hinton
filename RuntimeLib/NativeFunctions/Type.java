package org.hinton_lang.RuntimeLib.NativeFunctions;

import java.util.HashMap;

import org.hinton_lang.Interpreter.NativeType;
import org.hinton_lang.Interpreter.HintonFunctions.HintonCallable;
import org.hinton_lang.Interpreter.HintonString.HintonString;

public class Type implements NativeFunc {

    /**
     * Specifies the name of the function.
     */
    @Override
    public String getFuncName() {
        return "type";
    }

    /**
     * Specifies the function's body.
     */
    @Override
    public HintonCallable getFunc() {
        return new HintonCallable() {
            @Override
            public HintonString call(HashMap<Object, Object> arguments) {
                Object arg = arguments.get(0);

                if (arg instanceof NativeType) {
                    return new HintonString(((NativeType) arg).typeName());
                } else {
                    return new HintonString(arg.getClass().getName());
                }
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
                return "<HintonFunction: type>";
            }
        };
    }
}