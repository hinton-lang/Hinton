package org.hinton_lang.AbstractSyntaxTree;

import org.hinton_lang.Tokens.Token;

public class RealLiteral extends AST {
    public double value;

    public RealLiteral(Token token) {
        value = (double) token.value;
    }

    @Override
    public String toString() {
        return "RealLiteral{" +
                "value=" + value +
                '}';
    }
}
