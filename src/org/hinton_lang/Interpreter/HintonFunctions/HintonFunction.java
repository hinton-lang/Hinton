package org.hinton_lang.Interpreter.HintonFunctions;

import java.util.ArrayList;
import java.util.HashMap;

import org.hinton_lang.Parser.AST.Stmt;
import org.hinton_lang.Tokens.Token;
import org.hinton_lang.Errors.RuntimeError;
import org.hinton_lang.Interpreter.Interpreter;

/**
 * Represents a function declaration.
 */
public class HintonFunction extends HintonCallable {
    public Stmt.Function declaration;

    public HintonFunction(Interpreter interpret, Stmt.Function declaration) {
        super(declaration.name.lexeme, interpret, declaration);
        this.declaration = declaration;
    }

    public HintonFunction(String name, Interpreter interpret, Stmt.Function declaration) {
        super(name, interpret, declaration);
        this.declaration = declaration;
    }

    /**
     * Creates a scope and executes the function on every function call.
     */
    @Override
    public Object call(Token token, HashMap<Object, Object> arguments) {
        Object[] argsList = arguments.keySet().toArray();
        ArrayList<String> alreadySet = new ArrayList<>();

        for (int j = 0, argsListLength = argsList.length; j < argsListLength; j++) {
            Object arg = argsList[j];
            Object val = arguments.get(arg);

            if (arg instanceof Integer) {
                int i = (int) arg;
                alreadySet.add(this.parameters.get(i).name.lexeme);
                this.environment.assign(this.parameters.get(i).name, val);
            } else {
                Token t = (Token) arg;

                if (alreadySet.contains(t.lexeme)) {
                    throw new RuntimeError(t, this.declaration.name.lexeme
                            + "() received multiple values for argument '" + t.lexeme + "'.");
                } else {
                    if (environment.contains(t)) {
                        alreadySet.add(t.lexeme);
                        environment.assign(t, val);
                    } else {
                        throw new RuntimeError(t, "Unexpected named argument '" + t.lexeme + "'.");
                    }
                }
            }
        }

        return this.executeFunc();
    }

    /**
     * String representation.
     */
    @Override
    public String toString() {
        return "<Function " + declaration.name.lexeme + ">";
    }
}