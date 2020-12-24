package org.hinton_lang.Parser.AST;

import java.util.List;
import org.hinton_lang.Tokens.Token;

public abstract class Expr {

    public interface Visitor<R> {
        public R visitAssignExpr(Assign expr);

        public R visitBinaryExpr(Binary expr);

        public R visitLambdaExpr(Lambda expr);

        public R visitCallExpr(Call expr);

        public R visitMemberAccessExpr(MemberAccess expr);

        public R visitMemberSetterExpr(MemberSetter expr);

        public R visitGroupingExpr(Grouping expr);

        public R visitLiteralExpr(Literal expr);

        public R visitLogicalExpr(Logical expr);

        public R visitArrayExpr(Array expr);

        public R visitIndexingExpr(Indexing expr);

        public R visitArrayItemSetterExpr(ArrayItemSetter expr);

        public R visitUnaryExpr(Unary expr);

        public R visitVariableExpr(Variable expr);
    }

    public abstract <R> R accept(Visitor<R> visitor);

    public static class Assign extends Expr {
        public final Token name;
        public final Expr value;

        public Assign(Token name, Expr value) {
            this.name = name;
            this.value = value;
        }

        @Override
        public <R> R accept(Visitor<R> visitor) {
            return visitor.visitAssignExpr(this);
        }
    }

    public static class Binary extends Expr {
        public final Expr left;
        public final Token operator;
        public final Expr right;

        public Binary(Expr left, Token operator, Expr right) {
            this.left = left;
            this.operator = operator;
            this.right = right;
        }

        @Override
        public <R> R accept(Visitor<R> visitor) {
            return visitor.visitBinaryExpr(this);
        }
    }

    public static class Lambda extends Expr {
        public final List<Token> params;
        public final List<Stmt> body;

        public Lambda(List<Token> params, List<Stmt> body) {
            this.params = params;
            this.body = body;
        }

        @Override
        public <R> R accept(Visitor<R> visitor) {
            return visitor.visitLambdaExpr(this);
        }
    }

    public static class Call extends Expr {
        public final Expr callee;
        public final Token paren;
        public final List<Expr> arguments;

        public Call(Expr callee, Token paren, List<Expr> arguments) {
            this.callee = callee;
            this.paren = paren;
            this.arguments = arguments;
        }

        @Override
        public <R> R accept(Visitor<R> visitor) {
            return visitor.visitCallExpr(this);
        }
    }

    public static class MemberAccess extends Expr {
        public final Expr object;
        public final Token name;

        public MemberAccess(Expr object, Token name) {
            this.object = object;
            this.name = name;
        }

        @Override
        public <R> R accept(Visitor<R> visitor) {
            return visitor.visitMemberAccessExpr(this);
        }
    }

    public static class MemberSetter extends Expr {
        public final Expr object;
        public final Token name;
        public final Expr value;

        public MemberSetter(Expr object, Token name, Expr value) {
            this.object = object;
            this.name = name;
            this.value = value;
        }

        @Override
        public <R> R accept(Visitor<R> visitor) {
            return visitor.visitMemberSetterExpr(this);
        }
    }

    public static class Grouping extends Expr {
        public final Expr expression;

        public Grouping(Expr expression) {
            this.expression = expression;
        }

        @Override
        public <R> R accept(Visitor<R> visitor) {
            return visitor.visitGroupingExpr(this);
        }
    }

    public static class Literal extends Expr {
        public final Object value;

        public Literal(Object value) {
            this.value = value;
        }

        @Override
        public <R> R accept(Visitor<R> visitor) {
            return visitor.visitLiteralExpr(this);
        }
    }

    public static class Logical extends Expr {
        public final Expr left;
        public final Token operator;
        public final Expr right;

        public Logical(Expr left, Token operator, Expr right) {
            this.left = left;
            this.operator = operator;
            this.right = right;
        }

        @Override
        public <R> R accept(Visitor<R> visitor) {
            return visitor.visitLogicalExpr(this);
        }
    }

    public static class Array extends Expr {
        public final List<Expr> expressions;

        public Array(List<Expr> expressions) {
            this.expressions = expressions;
        }

        @Override
        public <R> R accept(Visitor<R> visitor) {
            return visitor.visitArrayExpr(this);
        }
    }

    public static class Indexing extends Expr {
        public final Expr arr;
        public final Expr index;

        public Indexing(Expr arr, Expr index) {
            this.arr = arr;
            this.index = index;
        }

        @Override
        public <R> R accept(Visitor<R> visitor) {
            return visitor.visitIndexingExpr(this);
        }
    }

    public static class ArrayItemSetter extends Expr {
        public final Expr.Indexing target;
        public final Expr value;

        public ArrayItemSetter(Expr.Indexing target, Expr value) {
            this.target = target;
            this.value = value;
        }

        @Override
        public <R> R accept(Visitor<R> visitor) {
            return visitor.visitArrayItemSetterExpr(this);
        }
    }

    public static class Unary extends Expr {
        public final Token operator;
        public final Expr right;

        public Unary(Token operator, Expr right) {
            this.operator = operator;
            this.right = right;
        }

        @Override
        public <R> R accept(Visitor<R> visitor) {
            return visitor.visitUnaryExpr(this);
        }
    }

    public static class Variable extends Expr {
        public final Token name;

        public Variable(Token name) {
            this.name = name;
        }

        @Override
        public <R> R accept(Visitor<R> visitor) {
            return visitor.visitVariableExpr(this);
        }
    }
}
