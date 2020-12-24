package org.hinton_lang.RuntimeLib.NativeFunctions;

import java.util.List;

import org.hinton_lang.Errors.RuntimeError;
import org.hinton_lang.Interpreter.HintonFunctions.HintonCallable;
import org.hinton_lang.Interpreter.HintonInteger.HintonInteger;
import org.hinton_lang.Interpreter.HintonString.HintonString;
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
            public HintonInteger call(Interpreter interpreter, List<Object> arguments) {

                // Checks that the passed argument is a Hinton String
                String strInt;
                if (arguments.get(0) instanceof HintonString) {
                    strInt = ((HintonString) arguments.get(0)).getRaw();
                } else {
                    throw new RuntimeError("Cannot cast \"" + arguments.get(0) + "\" to integer");
                }

                // Tries to convert the string to an integer, if not possible, throw a
                // RuntimeError.
                try {
                    return new HintonInteger(Integer.parseInt(strInt));
                } catch (NumberFormatException e) {
                    throw new RuntimeError("Cannot cast \"" + arguments.get(0) + "\" to integer");
                }
            }

            @Override
            public String toString() {
                return "<HintonFunction: int>";
            }
        };
    }
}
