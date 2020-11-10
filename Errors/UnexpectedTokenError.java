package org.hinton_lang.Errors;

import org.hinton_lang.Tokens.Token;

public class UnexpectedTokenError extends Error {
    /**
	 * Serial Version ID
	 */
	private static final long serialVersionUID = -1061877694785216885L;

    
    /**
     * The token was not expected
     * @param token The current token
     */
	public UnexpectedTokenError(Token token) {
        super("Unexpected Token '" + token.text + "' on line " + token.linePos + ":" + token.columnPos);
    }
}
