package org.hinton_lang.RuntimeLib.NativeFunctions;

import java.util.List;
import java.util.Scanner;

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
                Scanner s = new Scanner(System.in);
                System.out.println(Helper.stringify(arguments.get(0)));
                String usrInput = s.nextLine();
                s.close();
                return usrInput;
            }

            @Override
            public String toString() {
                return "<HintonFunction: input>";
            }
        };
    }
}
