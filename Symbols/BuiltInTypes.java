package org.hinton_lang.Symbols;

public enum BuiltInTypes {
    INT("Int"),
    REAL("Real"),
    CHAR("Char"),
    STRING("String"),
    BOOLEAN("Bool"),

    // Other Types
    ANY("Any"),
    NULL("Null"),
    FUNC("Function"),
    VOID("Void");

    private BuiltInTypes(String t) { }
}
