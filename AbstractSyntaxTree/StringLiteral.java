package org.hinton_lang.AbstractSyntaxTree;

import org.hinton_lang.Tokens.Token;

public class StringLiteral extends AST {
    public Token token;

	public StringLiteral(Token token) {
		this.token = token;
    }

    @Override
    public String toString() {
        return "StringLiteral{" +
                "token=" + token +
                '}';
    }
}
