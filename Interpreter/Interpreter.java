package org.hinton_lang.Interpreter;

import java.util.List;

import org.hinton_lang.Parser.AST.*;
import org.hinton_lang.Hinton;
import org.hinton_lang.Errors.RuntimeError;
import org.hinton_lang.Envornment.Environment;
import org.hinton_lang.Parser.AST.Stmt;
import org.hinton_lang.Tokens.TokenType;

public class Interpreter implements Expr.Visitor<Object>, Stmt.Visitor<Void> {
    // Used to store variables
    public Environment environment = new Environment();

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

    @Override
    public Object visitLogicalExpr(Expr.Logical expr) {
        Object left = evaluate(expr.left);

        if (expr.operator.type == TokenType.LOGICAL_OR) {
            if (EvalUnaryExpr.isTruthy(left))
                return left;
        } else {
            if (!EvalUnaryExpr.isTruthy(left))
                return left;
        }

        return evaluate(expr.right);
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
     * Visits a block statement.
     */
    @Override
    public Void visitBlockStmt(Stmt.Block stmt) {
        executeBlock(stmt.statements, new Environment(environment));
        return null;
    }

    /**
     * Executes the contents of a block statement.
     * 
     * @param statements  The statements contained within the block.
     * @param environment The new environment for this block.
     */
    private void executeBlock(List<Stmt> statements, Environment environment) {
        Environment previous = this.environment;
        try {
            this.environment = environment;

            for (Stmt statement : statements) {
                execute(statement);
            }
        } finally {
            this.environment = previous;
        }
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

    @Override
    public Void visitIfStmt(Stmt.If stmt) {
        if (EvalUnaryExpr.isTruthy(evaluate(stmt.condition))) {
            execute(stmt.thenBranch);
        } else if (stmt.elseBranch != null) {
            execute(stmt.elseBranch);
        }
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
     * Visits a variable statement.
     */
    @Override
    public Void visitVarStmt(Stmt.Var stmt) {
        Object value = null;
        if (stmt.initializer != null) {
            value = evaluate(stmt.initializer);
        }

        environment.defineVar(stmt.name.lexeme, value);
        return null;
    }

    /**
     * Visits a constant statement.
     */
    @Override
    public Void visitConstStmt(Stmt.Const stmt) {
        Object value = evaluate(stmt.initializer);

        environment.defineConst(stmt.name.lexeme, value);
        return null;
    }

    /**
     * Visits an assignment expression.
     */
    @Override
    public Object visitAssignExpr(Expr.Assign expr) {
        Object value = evaluate(expr.value);
        environment.assign(expr.name, value);
        return value;
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
     * Visits a variable expression.
     */
    @Override
    public Object visitVariableExpr(Expr.Variable expr) {
        return environment.get(expr.name);
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