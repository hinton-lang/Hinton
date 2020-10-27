package org.hinton_lang.AbstractSyntaxTree;

import org.hinton_lang.Tokens.Token;

public class Identifier extends AST {
    public Token token;

	public Identifier(Token token) {
		this.token = token;
    }

    @Override
    public String toString() {
        return "Identifier{" +
                "token=" + token +
                '}';
    }
}
