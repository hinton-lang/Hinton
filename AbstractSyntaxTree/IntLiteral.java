package org.hinton_lang.AbstractSyntaxTree;

import org.hinton_lang.Tokens.Token;

public class IntLiteral extends AST {
    public int value;

    public IntLiteral(Token token) {
        value = (int) token.value;
    }

    @Override
    public String toString() {
        return "IntLiteral{" +
                "value=" + value +
                '}';
    }
}
