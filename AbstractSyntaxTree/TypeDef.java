package org.hinton_lang.AbstractSyntaxTree;

import org.hinton_lang.Symbols.BuiltInTypes;

public class TypeDef extends AST {
    public BuiltInTypes type;

	public TypeDef(BuiltInTypes type) {
		this.type = type;
    }

    @Override
    public String toString() {
        return "TypeDef{" +
                "type=" + type +
                '}';
    }
}
