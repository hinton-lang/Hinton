package org.hinton_lang.AbstractSyntaxTree;

import org.hinton_lang.Symbols.AssignmentType;

public class ReAssignment extends AST {
    public AssignmentType type;
	public AST left;
	public AST right;

    public ReAssignment(AssignmentType type, VarDeclaration left, AST right) {
		this.type = type;
		this.left = left;
		this.right = right;
    }

	@Override
	public String toString() {
		return "ReAssignment{" +
				"type=" + type +
				", left=" + left +
				", right=" + right +
				'}';
	}
}
