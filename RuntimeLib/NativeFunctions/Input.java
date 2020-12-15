package org.hinton_lang.RuntimeLib.NativeFunctions;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.util.List;

import org.hinton_lang.Helper.Helper;
import org.hinton_lang.Interpreter.HintonCallable;
import org.hinton_lang.Interpreter.Interpreter;

/**
 * Native Hinton function for reading user input from the command prompt.
 */
public class Input implements NativeFunc {

    /**
     * Specifies the name of the function.
     */
    @Override
    public String getFuncName() {
        return "input";
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
            public String call(Interpreter interpreter, List<Object> arguments) {
                InputStreamReader input = new InputStreamReader(System.in);
                BufferedReader reader = new BufferedReader(input);

                System.out.print(Helper.stringify(arguments.get(0)));

                try {
                    String usrInput = reader.readLine();
                    return usrInput;
                } catch (IOException e) {
                    return "";
                }
            }

            @Override
            public String toString() {
                return "<HintonFunction: input>";
            }
        };
    }
}
