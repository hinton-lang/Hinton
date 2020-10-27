package org.hinton_lang.Errors;

import org.hinton_lang.Tokens.Token;


public class UnexpectedTokenError extends Error {
    public UnexpectedTokenError(Token token) {
        super("Unexpected Token '" + token.text + "' on line " + token.linePos + ":" + token.columnPos);
    }
}
