package org.hinton_lang.Interpreter;

import org.hinton_lang.Tokens.Token;
import org.hinton_lang.Errors.RuntimeError;

public class EvalBinaryExpr {

    /**
     * Checks that the two operands hold numeric value.
     * 
     * @param operator The operator to be applied on the operands.
     * @param left     The left operand.
     * @param right    The right operand.
     */
    private static void checkNumberOperands(Token operator, Object left, Object right) {
        if ((left instanceof Double || left instanceof Integer)
                && (right instanceof Double || right instanceof Integer))
            return;

        throw new RuntimeError(operator, "Operands must be numbers.");
    }

    /**
     * Checks that the two operands hold equals values.
     * 
     * @param left  The left operand.
     * @param right The right operand.
     */
    private static boolean isEqual(Object left, Object right) {
        if (left == null && right == null)
            return true;
        if (left == null)
            return false;

        return left.equals(right);
    }

    /**
     * Evaluates an addition expression.
     * 
     * @param opr   The addition operator token.
     * @param left  The left operand.
     * @param right The right operand.
     */
    public static Object evalAddition(Token opr, Object left, Object right) {
        // Addition or reals
        if (left instanceof Double && right instanceof Double) {
            return (double) left + (double) right;
        }
        // Additions of integers
        if (left instanceof Integer && right instanceof Integer) {
            return (int) left + (int) right;
        }
        // Addition of reals and integers
        if (left instanceof Integer && right instanceof Double) {
            return (int) left + (double) right;
        }
        if (left instanceof Double && right instanceof Integer) {
            return (double) left + (int) right;
        }

        // Addition of two strings
        if (left instanceof String && right instanceof String) {
            return (String) left + (String) right;
        }

        // Addition of Strings and numbers
        if (left instanceof String && right instanceof Double) {
            return (String) left + (double) right;
        }
        if (left instanceof String && right instanceof Integer) {
            return (String) left + (int) right;
        }
        if (left instanceof Double && right instanceof String) {
            return (double) left + (String) right;
        }
        if (left instanceof Integer && right instanceof String) {
            return (int) left + (String) right;
        }

        throw new RuntimeError(opr, "Operands must be numbers or strings.");
    }

    /**
     * Evaluates a subtraction expression.
     * 
     * @param opr   The subtraction operator token.
     * @param left  The left operand.
     * @param right The right operand.
     */
    public static Object evalSubtraction(Token opr, Object left, Object right) {
        checkNumberOperands(opr, left, right);

        if (left instanceof Double && right instanceof Double) {
            return (double) left - (double) right;
        }
        if (left instanceof Double && right instanceof Integer) {
            return (double) left - (int) right;
        }
        if (left instanceof Integer && right instanceof Double) {
            return (int) left - (double) right;
        }

        return (int) left - (int) right;
    }

    /**
     * Evaluates a division expression.
     * 
     * @param opr   The division operator token.
     * @param left  The left operand.
     * @param right The right operand.
     */
    public static Object evalDivision(Token opr, Object left, Object right) {
        checkNumberOperands(opr, left, right);

        // Prevents division by zero
        if (right instanceof Integer && (int) right == 0) {
            throw new RuntimeError(opr, "Cannot divide by zero.");
        }
        if (right instanceof Double && (double) right == 0.0) {
            throw new RuntimeError(opr, "Cannot divide by zero.");
        }

        // If no division by zero, continue to execute the division
        if (left instanceof Double && right instanceof Double) {
            return (double) left / (double) right;
        }
        if (left instanceof Double && right instanceof Integer) {
            return (double) left / (int) right;
        }
        if (left instanceof Integer && right instanceof Double) {
            return (int) left / (double) right;
        }

        // In Hinton, division always evaluates to a real number.
        return (double) ((int) left) / (double) ((int) right);
    }

    /**
     * Evaluates a multiplication expression.
     * 
     * @param opr   The multiplication operator token.
     * @param left  The left operand.
     * @param right The right operand.
     */
    public static Object evalMultiplication(Token opr, Object left, Object right) {
        checkNumberOperands(opr, left, right);

        if (left instanceof String)
            return ((String) left).repeat((int) right);
        if (right instanceof String)
            return ((String) right).repeat((int) left);

        if (left instanceof Double && right instanceof Double) {
            return (double) left * (double) right;
        }
        if (left instanceof Double && right instanceof Integer) {
            return (double) left * (int) right;
        }
        if (left instanceof Integer && right instanceof Double) {
            return (int) left * (double) right;
        }

        return (int) left * (int) right;
    }

    /**
     * Evaluates a modulus expression.
     * 
     * @param opr   The modulus operator token.
     * @param left  The left operand.
     * @param right The right operand.
     */
    public static Object evalModulus(Token opr, Object left, Object right) {
        checkNumberOperands(opr, left, right);

        if (left instanceof Double && right instanceof Double) {
            return (double) left % (double) right;
        }
        if (left instanceof Double && right instanceof Integer) {
            return (double) left % (int) right;
        }
        if (left instanceof Integer && right instanceof Double) {
            return (int) left % (double) right;
        }

        return (int) left % (int) right;
    }

    /**
     * Evaluates an exponentiation expression.
     * 
     * @param opr   The exponentiation operator token.
     * @param left  The left operand.
     * @param right The right operand.
     */
    public static Object evalExponent(Token opr, Object left, Object right) {
        checkNumberOperands(opr, left, right);

        if (left instanceof Double && right instanceof Double) {
            return Math.pow((double) left, (double) right);
        }
        if (left instanceof Double && right instanceof Integer) {
            return Math.pow((double) left, (int) right);
        }
        if (left instanceof Integer && right instanceof Double) {
            return Math.pow((int) left, (double) right);
        }

        return Math.pow((int) left, (int) right);
    }

    /**
     * Evaluates a greater-than expression.
     * 
     * @param opr   The greater-than operator token.
     * @param left  The left operand.
     * @param right The right operand.
     */
    public static Object evalGreaterThan(Token opr, Object left, Object right) {
        checkNumberOperands(opr, left, right);

        if (left instanceof Boolean) {
            left = (Boolean) left ? 1 : 0;
        }
        if (right instanceof Boolean) {
            right = (Boolean) right ? 1 : 0;
        }

        if (left instanceof Double && right instanceof Double) {
            return (double) left > (double) right;
        }
        if (left instanceof Double && right instanceof Integer) {
            return (double) left > (int) right;
        }
        if (left instanceof Integer && right instanceof Double) {
            return (int) left > (double) right;
        }

        return (int) left > (int) right;
    }

    /**
     * Evaluates a greater-than-or-equal expression.
     * 
     * @param opr   The greater-than-or-equal operator token.
     * @param left  The left operand.
     * @param right The right operand.
     */
    public static Object evalGreaterThanEqual(Token opr, Object left, Object right) {
        checkNumberOperands(opr, left, right);

        if (left instanceof Boolean) {
            left = (Boolean) left ? 1 : 0;
        }
        if (right instanceof Boolean) {
            right = (Boolean) right ? 1 : 0;
        }

        if (left instanceof Double && right instanceof Double) {
            return (double) left >= (double) right;
        }
        if (left instanceof Double && right instanceof Integer) {
            return (double) left >= (int) right;
        }
        if (left instanceof Integer && right instanceof Double) {
            return (int) left >= (double) right;
        }

        return (int) left >= (int) right;
    }

    /**
     * Evaluates a less-than expression.
     * 
     * @param opr   The less-than operator token.
     * @param left  The left operand.
     * @param right The right operand.
     */
    public static Object evalLessThan(Token opr, Object left, Object right) {
        checkNumberOperands(opr, left, right);

        if (left instanceof Boolean) {
            left = (Boolean) left ? 1 : 0;
        }
        if (right instanceof Boolean) {
            right = (Boolean) right ? 1 : 0;
        }

        if (left instanceof Double && right instanceof Double) {
            return (double) left < (double) right;
        }
        if (left instanceof Double && right instanceof Integer) {
            return (double) left < (int) right;
        }
        if (left instanceof Integer && right instanceof Double) {
            return (int) left < (double) right;
        }

        return (int) left < (int) right;
    }

    /**
     * Evaluates a less-than-or-equal expression.
     * 
     * @param opr   The less-than-or-equal operator token.
     * @param left  The left operand.
     * @param right The right operand.
     */
    public static Object evalLessThanEqual(Token opr, Object left, Object right) {
        checkNumberOperands(opr, left, right);

        if (left instanceof Boolean) {
            left = (Boolean) left ? 1 : 0;
        }
        if (right instanceof Boolean) {
            right = (Boolean) right ? 1 : 0;
        }

        if (left instanceof Double && right instanceof Double) {
            return (double) left <= (double) right;
        }
        if (left instanceof Double && right instanceof Integer) {
            return (double) left <= (int) right;
        }
        if (left instanceof Integer && right instanceof Double) {
            return (int) left <= (double) right;
        }

        return (int) left <= (int) right;
    }

    /**
     * Evaluates an equality expression.
     * 
     * @param opr   The equality operator token.
     * @param left  The left operand.
     * @param right The right operand.
     */
    public static Object evalEquals(Token opr, Object left, Object right) {
        if (left instanceof Boolean) {
            left = (Boolean) left ? 1 : 0;
        }
        if (right instanceof Boolean) {
            right = (Boolean) right ? 1 : 0;
        }
        return isEqual(left, right);
    }

    /**
     * Evaluates an inequality expression.
     * 
     * @param opr   The inequality operator token.
     * @param left  The left operand.
     * @param right The right operand.
     */
    public static Object evalNotEquals(Token opr, Object left, Object right) {
        if (left instanceof Boolean) {
            left = (Boolean) left ? 1 : 0;
        }
        if (right instanceof Boolean) {
            right = (Boolean) right ? 1 : 0;
        }
        return !isEqual(left, right);
    }
}
