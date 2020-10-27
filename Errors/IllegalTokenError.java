package org.hinton_lang.Errors;

import org.hinton_lang.Tokens.Token;

public class IllegalTokenError extends Error {
	public IllegalTokenError(Token token) {
        super("Unexpected Illegal Token '" + token.text + "' on line " + token.linePos + ":" + token.columnPos);
    }
}
