package org.hinton_lang.Symbols;

public enum BuiltInTypes {
    INT("Int"),
    REAL("Real"),
    CHAR("Char"),
    STRING("String"),
    BOOLEAN("Bool"),

    // Other Types
    ANY("any"),
    NULL("null"),
    FUNC("Function"),
    VOID("VOID");

    private BuiltInTypes(String t) { }
}
