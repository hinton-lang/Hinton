package org.hinton_lang.Interpreter;

import org.hinton_lang.Parser.AST.Stmt;

/**
 * Represents a class member. Contains all the information necessary to declare
 * public or private, static, or final functions and fields.
 */
public class ClassMember implements Cloneable {
    public boolean isPrivate;
    public boolean isStatic;
    public boolean isFinal;
    public Stmt member;
    public Object value;

    /**
     * Represents a class member.
     * 
     * @param member The class member.
     * @param value  The value of the member.
     */
    public ClassMember(Stmt.ClassMember member, Object value) {
        if (member.member instanceof Stmt.Function) {
            this.isFinal = true;
        } else {
            if (member.member instanceof Stmt.Field) {
                this.isFinal = ((Stmt.Field) member.member).isFinal;
            }
        }

        this.member = member;
        this.isPrivate = member.isPrivate;
        this.isStatic = member.isStatic;
        this.value = value;
    }

    /**
     * Creates a clone of the class member.
     */
    public ClassMember clone() throws CloneNotSupportedException {
        return (ClassMember) super.clone();
    }
}
