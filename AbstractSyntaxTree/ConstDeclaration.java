package org.hinton_lang.AbstractSyntaxTree;

public class ConstDeclaration extends AST {
    public Identifier left;
	public TypeDef right;

	public ConstDeclaration(Identifier left, TypeDef right) {
		this.left = left;
		this.right = right;
    }

	@Override
	public String toString() {
		return "ConstDeclaration{" +
				"left=" + left +
				", right=" + right +
				'}';
	}
}
