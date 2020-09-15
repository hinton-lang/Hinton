package com.wci.frontend;

public abstract class Scanner {
    protected Source source;
    private Token currentToken;

    public Scanner(Source source) {
        this.source = source;
    }

    /**
     * Return the current token.
     * @return the current token.
     */
    public Token currentToken() {
        return currentToken;
    }


    /**
     * Return the next token from the source.
     * @return the next token.
     * @throws Exception if an error ocurred.
     */
    public Token getNextToken() throws Exception {
        currentToken = extractToken();
        return currentToken;
    }


    /**
     * Do the actual work of extracting and returning the next token from
     * the source. Implemented by scanner subclass.
     * @return the next token.
     * @throws Exception if an error occurred.
     */
    protected abstract Token extractToken() throws Exception;


    /**
     * Call the source's currentChar() method.
     * @return the current character from the source.
     * @throws Exception if an error occurred.
     */
    public char currentChar() throws Exception {
        return source.currentChar();
    }


    /**
     * Call the source's nextChar() method.
     * @return the next character from the source.
     * @throws Exception if an error occurred.
     */
    public char nextChar() throws Exception {
        return source.nextChar();
    }
}