package org.hinton_lang.Interpreter.HintonArrays;

import java.util.ArrayList;
import java.util.List;

import org.hinton_lang.Interpreter.HintonFunctions.HintonCallable;
import org.hinton_lang.Interpreter.HintonFunctions.HintonLambda;
import org.hinton_lang.Parser.AST.Expr;
import org.hinton_lang.Parser.AST.Stmt;
import org.hinton_lang.Errors.RuntimeError;
import org.hinton_lang.Interpreter.HintonBoolean.HintonBoolean;

/**
 * Method for looping through the items of an array.
 */
public class ArrayForEach implements HintonCallable {
    ArrayList<Object> arr;

    public ArrayForEach(ArrayList<Object> arr) {
        this.arr = arr;
    }

    @Override
    public Object call(List<Object> arguments) {
        HintonLambda lambda = (HintonLambda) arguments.get(0);

        // The callback accepted by HintonArray.forEach can only have
        // a max of two parameters, namely, item and index respectively.
        if (lambda.minArity() < 0 || lambda.maxArity() > 2) {
            Stmt.Parameter extraParam = lambda.parameters.get(2);
            throw new RuntimeError(extraParam.name,
                    "[Array].forEach accepts up to 2 parameter, got " + lambda.minArity() + " instead.");
        }

        for (int i = 0; i < this.arr.size(); i++) {
            Object item = this.arr.get(i);
            ArrayList<Object> args = new ArrayList<>();

            // Adds the array item to the callback's arguments
            if (item instanceof Expr) {
                args.add(lambda.interpreter.evaluate((Expr) item));
            } else {
                args.add(item);
            }

            // Adds the item's index to the callback's arguments
            args.add(i);

            // executes the callback
            Object c = lambda.call(args);

            // We break the loop if the programmer returns `false`
            // within the .forEach callback.
            if (c instanceof HintonBoolean && !((HintonBoolean) c).getRaw()) {
                break;
            }
        }

        return null;
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
        return "<Function: [Array].forEach>";
    }
}
