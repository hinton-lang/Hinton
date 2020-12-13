package org.hinton_lang.Parser.AST;

import java.util.List;
import org.hinton_lang.Tokens.Token;

public abstract class Stmt {

    public interface Visitor<R> {
        public R visitExpressionStmt(Expression stmt);
        public R visitPrintStmt(Print stmt);
    }

    public abstract <R> R accept(Visitor<R> visitor);


    public static class Expression extends Stmt {
        public final Expr expression;

        public Expression(Expr expression) {
            this.expression = expression;
        }

        @Override
        public <R> R accept(Visitor<R> visitor) {
            return visitor.visitExpressionStmt(this);
        }
    }


    public static class Print extends Stmt {
        public final Expr expression;

        public Print(Expr expression) {
            this.expression = expression;
        }

        @Override
        public <R> R accept(Visitor<R> visitor) {
            return visitor.visitPrintStmt(this);
        }
    }
}
