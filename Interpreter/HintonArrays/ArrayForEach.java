package org.hinton_lang.Interpreter.HintonArrays;

import java.util.ArrayList;
import java.util.List;

import org.hinton_lang.Interpreter.HintonFunctions.HintonCallable;
import org.hinton_lang.Interpreter.HintonFunctions.HintonLambda;
import org.hinton_lang.Parser.AST.Expr;
import org.hinton_lang.Tokens.Token;
import org.hinton_lang.Errors.RuntimeError;
import org.hinton_lang.Interpreter.Interpreter;

/**
 * Method for looping through the items of an array.
 */
public class ArrayForEach implements HintonCallable {
    ArrayList<Object> arr;

    public ArrayForEach(ArrayList<Object> arr) {
        this.arr = arr;
    }

    @Override
    public Object call(Interpreter interpreter, List<Object> arguments) {
        HintonLambda lambda = (HintonLambda) arguments.get(0);

        // The callback accepted by HintonArray.forEach can only have
        // a max of two parameters, namely, item and index respectively.
        if (lambda.arity() > 2) {
            Token extraParam = lambda.declaration.params.get(2);
            throw new RuntimeError(extraParam,
                    "[Array].forEach accepts up to 2 parameter, got " + lambda.arity() + " instead.");
        }

        // TODO: Add support for return statements inside .forEach methods.
        // Having a return statement inside a `.forEach` loop breaks the loop.
        for (int i = 0; i < this.arr.size(); i++) {
            Object item = this.arr.get(i);
            ArrayList<Object> args = new ArrayList<>();

            // Adds the array item to the callback's arguments
            if (item instanceof Expr) {
                args.add(interpreter.evaluate((Expr) item));
            } else {
                args.add(item);
            }

            // Adds the item's index to the callback's arguments
            args.add(i);

            // executes the callback
            lambda.call(interpreter, args);
        }

        return null;
    }

    @Override
    public int arity() {
        return 1;
    }

    @Override
    public String toString() {
        return "<Function: [Array].forEach>";
    }
}
