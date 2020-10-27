package org.hinton_lang.AbstractSyntaxTree;

import java.util.ArrayList;

import org.hinton_lang.Helper.PrintAST;

/**
 * The AST's root of any program.
 */
public class Program extends AST {
    public ArrayList<AST> statements;
    private PrintAST ASTPrinter;

    public Program(ArrayList<AST> statements) {
        this.statements = statements;
        ASTPrinter = new PrintAST();
    }


    @Override
    public String toString() {
        return ASTPrinter.print(this);
    }
}
