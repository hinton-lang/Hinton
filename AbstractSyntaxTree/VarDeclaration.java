package org.hinton_lang.AbstractSyntaxTree;

public class VarDeclaration extends AST {
    public Identifier left;
	public TypeDef right;

	public VarDeclaration(Identifier left, TypeDef right) {
		this.left = left;
		this.right = right;
    }

	@Override
	public String toString() {
		return "VarDeclaration{" +
				"left=" + left +
				", right=" + right +
				'}';
	}
}
