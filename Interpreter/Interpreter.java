package org.hinton_lang.Interpreter;

import java.util.List;
import java.util.ArrayList;
import java.util.HashMap;

import org.hinton_lang.Parser.AST.*;
import org.hinton_lang.Hinton;
import org.hinton_lang.Errors.RuntimeError;
import org.hinton_lang.Envornment.DecType;
import org.hinton_lang.Envornment.Environment;
import org.hinton_lang.Parser.AST.Stmt;
import org.hinton_lang.Parser.AST.Stmt.Import;
import org.hinton_lang.RuntimeLib.RuntimeLib;
import org.hinton_lang.Tokens.Token;
import org.hinton_lang.Tokens.TokenType;

public class Interpreter implements Expr.Visitor<Object>, Stmt.Visitor<Void> {
    // Holds global functions and variables native to Hinton.
    public final Environment globals = new Environment();
    // Used to store variables
    public Environment environment = globals;
    // A table that holds the declared variables and their scope distance.
    private final HashMap<Expr, Integer> locals = new HashMap<>();

    public Interpreter() {
        // Attaches the native functions to the global scope
        RuntimeLib.nativeFunctions.forEach((fn) -> {
            globals.defineBuiltIn(fn.getFuncName(), fn.getFunc(), DecType.HINTON_FUNCTION);
        });
    }

    /**
     * Tells the interpreter how many scopes there are between the current scope and
     * the scope where the variable is defined.
     * 
     * @param expr  The variable expression.
     * @param depth The scope distance.
     */
    public void resolve(Expr expr, int depth) {
        locals.put(expr, depth);
    }

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
     * Computes the boolean value of the provided object.
     * 
     * @param object The object whose boolean value will be computed.
     * @return The boolean value of the provided object.
     */
    public static boolean isTruthy(Object object) {
        if (object == null)
            return false;
        if (object instanceof Integer && (int) object == 0)
            return false;
        if (object instanceof Double && (int) object == 0.0)
            return false;
        if (object instanceof Boolean)
            return (boolean) object;
        return true;
    }

    /**
     * Visits a literal expression.
     */
    @Override
    public Object visitLiteralExpr(Expr.Literal expr) {
        return expr.value;
    }

    /**
     * Visits a logical expression.
     */
    @Override
    public Object visitLogicalExpr(Expr.Logical expr) {
        Object left = evaluate(expr.left);

        if (expr.operator.type == TokenType.LOGICAL_OR) {
            if (isTruthy(left))
                return left;
        } else {
            if (!isTruthy(left))
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
    public void executeBlock(List<Stmt> statements, Environment environment) {
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

    /**
     * Visits a function declaration statement.
     */
    @Override
    public Void visitFunctionStmt(Stmt.Function stmt) {
        HintonFunction function = new HintonFunction(stmt, environment);
        environment.define(stmt.name, function, DecType.FUNCTION);
        return null;
    }

    /**
     * Visits a break statement.
     */
    @Override
    public Void visitBreakStmt(Stmt.Break stmt) throws Break {
        // We use a throw-statement to trace back all the
        // way to where the loop's body was executed.
        throw new Break();
    }

    /**
     * Visits a continue statement.
     */
    @Override
    public Void visitContinueStmt(Stmt.Continue stmt) throws Continue {
        // We use a throw-statement to trace back all the
        // way to where the loop's body was executed.
        throw new Continue();
    }

    /**
     * Visits a function declaration.
     */
    @Override
    public Void visitReturnStmt(Stmt.Return stmt) {
        Object value = null;
        if (stmt.value != null)
            value = evaluate(stmt.value);

        // We use a throw-statement to trace back all the
        // way to where the function's body was executed.
        throw new Return(value);
    }

    /**
     * Visits an if statement.
     */
    @Override
    public Void visitIfStmt(Stmt.If stmt) {
        if (isTruthy(evaluate(stmt.condition))) {
            execute(stmt.thenBranch);
        } else if (stmt.elseBranch != null) {
            execute(stmt.elseBranch);
        }
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

        environment.define(stmt.name, value, DecType.VARIABLE);
        return null;
    }

    /**
     * Visits a while statement.
     */
    @Override
    public Void visitWhileStmt(Stmt.While stmt) {
        while (isTruthy(evaluate(stmt.condition))) {
            try {
                execute(stmt.body);
            } catch (Continue c) {
                continue;
            } catch (Break b) {
                break;
            }
        }
        return null;
    }

    /**
     * Visits a constant statement.
     */
    @Override
    public Void visitConstStmt(Stmt.Const stmt) {
        Object value = evaluate(stmt.initializer);

        environment.define(stmt.name, value, DecType.CONSTANT);
        return null;
    }

    /**
     * Visits an assignment expression.
     */
    @Override
    public Object visitAssignExpr(Expr.Assign expr) {
        Object value = evaluate(expr.value);
        Integer distance = locals.get(expr);
        if (distance != null) {
            environment.assignAt(distance, expr.name, value);
        } else {
            globals.assign(expr.name, value);
        }
        return value;
    }

    /**
     * Visits a lambda function expression.
     */
    @Override
    public Object visitLambdaExpr(Expr.Lambda expr) {
        return new HintonLambda(expr, environment);
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
        return lookUpVariable(expr.name, expr);
    }

    /**
     * Looks up a variable at a certain distance.
     * 
     * @param name The name of the variable.
     * @param expr The variable expression.
     * @return The variable's value.
     */
    private Object lookUpVariable(Token name, Expr expr) {
        Integer distance = locals.get(expr);
        if (distance != null) {
            return environment.getAt(distance, name.lexeme);
        } else {
            return globals.get(name);
        }
    }

    /**
     * Visits a function call expression.
     */
    @Override
    public Object visitCallExpr(Expr.Call expr) {
        Object callee = evaluate(expr.callee);

        List<Object> arguments = new ArrayList<>();
        for (Expr argument : expr.arguments) {
            arguments.add(evaluate(argument));
        }

        if (!(callee instanceof HintonCallable)) {
            throw new RuntimeError(expr.paren, "Can only call functions and classes.");
        }

        HintonCallable function = (HintonCallable) callee;

        if (arguments.size() != function.arity()) {
            throw new RuntimeError(expr.paren,
                    "Expected " + function.arity() + " arguments but got " + arguments.size() + ".");
        }

        return function.call(this, arguments);
    }

    /**
     * Visits an array expression.
     */
    @Override
    public ArrayList<Object> visitArrayExpr(Expr.Array expr) {
        ArrayList<Object> ar = new ArrayList<>();

        for (int i = 0; i < expr.expressions.size(); i++) {
            Expr item = expr.expressions.get(i);

            if (item instanceof Expr.Literal || item instanceof Expr.Array) {
                ar.add(evaluate(item));
            } else {
                ar.add(item);
            }

        }

        return ar;
    }

    /**
     * Visits an array indexing expression.
     */
    @Override
    public Object visitArrayIndexingExpr(Expr.ArrayIndexing expr) {
        Object arr = evaluate(expr.arr);
        Object index = evaluate(expr.index);

        if (arr instanceof ArrayList) {
            ArrayList<Expr> arr1 = (ArrayList<Expr>) arr;

            int idx;
            if (index instanceof Integer) {
                idx = (int) index;
            } else {
                throw new RuntimeError("Cannot only use Integers as array index.");
            }

            // Support for negative indexing
            if (idx < 0)
                idx = arr1.size() + idx;

            // If even after adjusting for negative index the provided
            // index is out of range, we throw an error.
            if (idx < 0 || idx > (arr1.size() - 1)) {
                throw new RuntimeError("Array index out of range.");
            }

            // If the item in the array is an instance of an expression, then
            // we evaluate the expression. Otherwise we return the value.
            if (arr1.get(idx) instanceof Expr) {
                return evaluate(arr1.get(idx));
            } else {
                return arr1.get(idx);
            }
        } else {
            throw new RuntimeError("Can only index arrays.");
        }
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

    /**
     * Visits an import statement
     */
    @Override
    public Void visitImportStmt(Import stmts) {
        for (Stmt stmt : stmts.statements) {
            stmt.accept(this);
        }
        return null;
    }
}
