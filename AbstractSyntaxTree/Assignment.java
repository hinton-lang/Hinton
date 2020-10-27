package org.hinton_lang.AbstractSyntaxTree;

import org.hinton_lang.Symbols.AssignmentType;

public class Assignment extends AST {
    public AssignmentType type;
	public AST left;
	public AST right;

	public Assignment(AssignmentType type, ConstDeclaration left, AST right) {
		this.type = type;
		this.left = left;
		this.right = right;
    }

    public Assignment(AssignmentType type, VarDeclaration left, AST right) {
		this.type = type;
		this.left = left;
		this.right = right;
    }

    public Assignment(AssignmentType type, FuncDeclaration left, AST right) {
		this.type = type;
		this.left = left;
		this.right = right;
    }

	@Override
	public String toString() {
		return "Assignment{" +
				"type=" + type +
				", left=" + left +
				", right=" + right +
				'}';
	}
}
