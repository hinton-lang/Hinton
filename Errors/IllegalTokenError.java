package org.hinton_lang.Errors;

import org.hinton_lang.Tokens.Token;

public class IllegalTokenError extends Error {
	/**
	 * Serial Version ID
	 */
	private static final long serialVersionUID = -2503948893459935778L;

    
    /**
     * The token is illegal
     * @param token The current token
     */
	public IllegalTokenError(Token token) {
        super("Unexpected Illegal Token '" + token.lexeme + "' on line " + token.linePos + ":" + token.columnPos);
    }
}
