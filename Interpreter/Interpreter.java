package org.hinton_lang.Interpreter;

import java.util.List;

import org.hinton_lang.Parser.AST.*;
import org.hinton_lang.Hinton;
import org.hinton_lang.Errors.RuntimeError;

public class Interpreter implements Expr.Visitor<Object>, Stmt.Visitor<Void> {

    /**
     * Executes the given list of statements (program).
     * 
     * @param statements The list of statements that make up the program.
     */
    public void interpret(List<Stmt> statements) {
        try {
            for (Stmt statement : statements) {
                execute(statement);
            }
        } catch (RuntimeError error) {
            Hinton.runtimeError(error);
        }
    }

    /**
     * Converts the given object to a string for printing.
     * 
     * @param object The object to tbe converted.
     * @return The string version of the object.
     */
    private String stringify(Object object) {
        if (object == null)
            return "null";

        if (object instanceof Double) {
            String text = object.toString();
            if (text.endsWith(".0")) {
                text = text.substring(0, text.length() - 2);
            }
            return text;
        }

        return object.toString();
    }

    /**
     * Visits a literal expression.
     */
    @Override
    public Object visitLiteralExpr(Expr.Literal expr) {
        return expr.value;
    }

    /**
     * Visits a parenthesized expression.
     */
    @Override
    public Object visitGroupingExpr(Expr.Grouping expr) {
        return evaluate(expr.expression);
    }

    /**
     * Evaluates the given expression.
     * 
     * @param expr The expression to be evaluated.
     * @return The literal value obtained from the expression.
     */
    private Object evaluate(Expr expr) {
        return expr.accept(this);
    }

    /**
     * Executes the given statement.
     * 
     * @param stmt The statement to execute.
     */
    private void execute(Stmt stmt) {
        stmt.accept(this);
    }

    /**
     * Visits an expression statement.
     * 
     * @param stmt The statement to visit.
     * @return VOID.
     */
    @Override
    public Void visitExpressionStmt(Stmt.Expression stmt) {
        evaluate(stmt.expression);
        return null;
    }

    /**
     * Visits a print statement.
     * 
     * @param stmt The print statement to visit.
     * @return VOID.
     */
    @Override
    public Void visitPrintStmt(Stmt.Print stmt) {
        Object value = evaluate(stmt.expression);
        System.out.println(stringify(value));
        return null;
    }

    /**
     * Evaluates a unary expression.
     */
    @Override
    public Object visitUnaryExpr(Expr.Unary expr) {
        Object right = evaluate(expr.right);

        switch (expr.operator.type) {
            case LOGICAL_NOT:
                return EvalUnaryExpr.evalLogicNegation(right);
            case MINUS:
                return EvalUnaryExpr.evalNumericNegation(expr.operator, right);
            default:
                break;
        }

        // Unreachable.
        return null;
    }

    /**
     * Evaluates a binary expression.
     */
    @Override
    public Object visitBinaryExpr(Expr.Binary expr) {
        Object left = evaluate(expr.left);
        Object right = evaluate(expr.right);

        switch (expr.operator.type) {
            case MINUS:
                return EvalBinaryExpr.evalSubtraction(expr.operator, left, right);
            case DIV:
                return EvalBinaryExpr.evalDivision(expr.operator, left, right);
            case MULT:
                return EvalBinaryExpr.evalMultiplication(expr.operator, left, right);
            case PLUS:
                return EvalBinaryExpr.evalAddition(expr.operator, left, right);
            case MOD:
                return EvalBinaryExpr.evalModulus(expr.operator, left, right);
            case EXPO:
                return EvalBinaryExpr.evalExponent(expr.operator, left, right);
            case LOGICAL_OR:
                return (boolean) left || (boolean) right;
            case LOGICAL_AND:
                return (boolean) left && (boolean) right;
            case GREATER_THAN:
                return EvalBinaryExpr.evalGreaterThan(expr.operator, left, right);
            case GREATER_THAN_EQ:
                return EvalBinaryExpr.evalGreaterThanEqual(expr.operator, left, right);
            case LESS_THAN:
                return EvalBinaryExpr.evalLessThan(expr.operator, left, right);
            case LESS_THAN_EQ:
                return EvalBinaryExpr.evalLessThanEqual(expr.operator, left, right);
            case LOGICAL_EQ:
                return EvalBinaryExpr.evalEquals(expr.operator, left, right);
            case LOGICAL_NOT_EQ:
                return EvalBinaryExpr.evalNotEquals(expr.operator, left, right);
            default:
                break;
        }

        // Unreachable.
        return null;
    }
}