package org.hinton_lang.Errors;

import org.hinton_lang.Tokens.Token;

public class ParserError extends RuntimeException {
    public final Token token;

    /** Serial ID */
    private static final long serialVersionUID = 1L;

    public ParserError(Token token, String message) {
        super(message);
        this.token = token;
    }

    public ParserError(String message) {
        super(message);
        this.token = null;
    }
}
