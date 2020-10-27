package org.hinton_lang.AbstractSyntaxTree;

import org.hinton_lang.Tokens.Token;

// Node that holds a Binary Operator (e.g.: +, -, *, **, /, etc..).
public class BinaryOperator extends AST {

    public AST left;
	public Token operator;
	public AST right;

	public BinaryOperator(AST left, Token operator, AST right) {
		this.left = left;
		this.operator = operator;
		this.right = right;
    }

	@Override
	public String toString() {
		return "BinaryOperator{" +
				"left=" + left +
				", operator=" + operator +
				", right=" + right +
				'}';
	}
}
