package org.hinton_lang.AbstractSyntaxTree;

public class FuncDeclaration extends AST {
    public Identifier left;
	public TypeDef right;
	public Compound body;

	public FuncDeclaration(Identifier left, TypeDef right) {
		this.left = left;
		this.right = right;
    }

	@Override
	public String toString() {
		return "FuncDeclaration{" +
				"left=" + left +
				", returnTypeDef=" + right +
				", body=" + body +
				'}';
	}
}
