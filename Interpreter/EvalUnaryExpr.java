package org.hinton_lang.Interpreter;

import org.hinton_lang.Errors.RuntimeError;
import org.hinton_lang.Tokens.Token;

public class EvalUnaryExpr {

    /**
     * Checks whether the provided operand holds a valid numeric value.
     * 
     * @param operator The operator.
     * @param operand  The operand.
     */
    private static void checkNumberOperand(Token operator, Object operand) {
        if (operand instanceof Double || operand instanceof Integer || operand instanceof Boolean)
            return;
        throw new RuntimeError(operator, "Operand must be a number.");
    }

    /**
     * Evaluates the boolean negation of the provided object.
     * 
     * @param right The operand.
     * @return (Boolean) The negation of the provided object.
     */
    public static boolean evalLogicNegation(Object right) {
        return !Interpreter.isTruthy(right);
    }

    /**
     * Evaluates the numeric negation of the provided object.
     * 
     * @param opr   The operator to be applied on the operand.
     * @param right The operand.
     * @return (Number) The negation of the provided object.
     */
    public static Object evalNumericNegation(Token opr, Object right) {
        checkNumberOperand(opr, right);

        if (right instanceof Boolean && (Boolean) right == true)
            return -1;
        if (right instanceof Boolean && (Boolean) right == false)
            return 0;
        if (right instanceof Double)
            return -(double) right;

        return -(int) right;
    }
}
