package org.hinton_lang.AbstractSyntaxTree;

import org.hinton_lang.Tokens.Token;

public class BoolLiteral extends AST {
    public Token token;

	public BoolLiteral(Token token) {
		this.token = token;
    }

    @Override
    public String toString() {
        return "BoolLiteral{" +
                "token=" + token +
                '}';
    }
}
