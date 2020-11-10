package org.hinton_lang.Tokens;

import java.util.HashMap;


public class Token {
    public TokenType type;
    public int linePos;
    public int columnPos;
    public String text;
    public Object value;


    public Token(TokenType type, int linePos, int columnPos, String text, Object value) {
        this.type = type;
        this.linePos = linePos;
        this.columnPos = columnPos;
        this.text = text;
        this.value = value;
    }


    /**
     * String representation of the Token.
     * @return The string representation of the Token.
     */
    public String toString() {
        // The largest token's name length is 27
        String leftPad = " ".repeat(30 - type.name().length());
        return type.name() + ":" + leftPad + text;
    }


    /** Maps the keyword's names to their corresponding token types. */
    public static HashMap<String, TokenType> Keywords = new HashMap<>();
    
    /**
     * Static Constructor
     */
    static {
        // Control Flow
        // "while", "for", "loop" "in", "break", "continue",

        // // Logic flow and operators
        // "not", "equals", "is",
        Keywords.put("if", TokenType.IF_KEYWORD);
        Keywords.put("elif", TokenType.ELIF_KEYWORD);
        Keywords.put("else", TokenType.ELSE_KEYWORD);
        Keywords.put("and", TokenType.LOGICAL_AND);
        Keywords.put("or", TokenType.LOGICAL_OR);
        Keywords.put("not", TokenType.LOGICAL_NOT);
        Keywords.put("equals", TokenType.LOGICAL_EQUALS);
        Keywords.put("is", TokenType.LOGICAL_IS);
        Keywords.put("mod", TokenType.ARITHMETIC_MODULUS);

        // // Module and OOP (no 'protected' keyword)
        // "import", "export", "new", "class", "readonly", "public",
        // "private", "self", "instanceof", "abstract", "implements",
        // "extends", "init", "static",
        

        // Static Types
        Keywords.put("Int", TokenType.INTEGER_TYPE);
        Keywords.put("Real", TokenType.REAL_TYPE);
        Keywords.put("Char", TokenType.CHARACTER_TYPE);
        Keywords.put("String", TokenType.STRING_TYPE);
        Keywords.put("Bool", TokenType.BOOLEAN_TYPE);
        Keywords.put("Dict", TokenType.DICTIONARY_TYPE);
        Keywords.put("Set", TokenType.SET_TYPE);
        Keywords.put("Function", TokenType.FUNCTION_TYPE);
        Keywords.put("Void", TokenType.VOID_TYPE);
        Keywords.put("Any", TokenType.ANY_TYPE);
        Keywords.put("Null", TokenType.NULL_TYPE);

        // Keyword Literals
        Keywords.put("true", TokenType.BOOLEAN_LITERAL_TRUE);
        Keywords.put("false", TokenType.BOOLEAN_LITERAL_FALSE);
        Keywords.put("null", TokenType.NULL_LITERAL);


        // Other
        // "async", "await", "as"

        // Declarations
        // "return", "yield", "enum", "define",
        Keywords.put("let", TokenType.LET_KEYWORD);
        Keywords.put("const", TokenType.CONST_KEYWORD);
        Keywords.put("func", TokenType.FUNC_KEYWORD);
    }
}


;
