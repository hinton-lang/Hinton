package org.hinton_lang.Errors;

import org.hinton_lang.Tokens.Token;

public class RuntimeError extends RuntimeException {
    public final Token token;

    /** Serial ID */
    private static final long serialVersionUID = -7660344298236100497L;

    public RuntimeError(Token token, String message) {
        super(message);
        this.token = token;
    }
}