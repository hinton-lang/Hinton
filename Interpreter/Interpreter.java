package org.hinton_lang.Interpreter;

import org.hinton_lang.Parser.AST.*;

public class Interpreter implements Expr.Visitor<Object> {

    @Override
    public Object visitLiteralExpr(Expr.Literal expr) {
        return expr.value;
    }
}