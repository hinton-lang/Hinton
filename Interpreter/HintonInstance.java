package org.hinton_lang.Interpreter;

import java.util.HashMap;

import org.hinton_lang.Errors.RuntimeError;
import org.hinton_lang.Parser.AST.Stmt;
import org.hinton_lang.Tokens.Token;

/**
 * Represents a class instance.
 */
public class HintonInstance {
    // The class from which the instance was created
    private HintonClass klass;
    // The members of this instance.
    // Each class instance gets its own copy of the original
    // members declared in the class definition.
    private final HashMap<String, ClassMember> instanceMembers;

    /**
     * Represents a class instance.
     * 
     * @param klass The class whose instance has been created.
     */
    public HintonInstance(HintonClass klass, HashMap<String, ClassMember> instanceMembers) {
        this.klass = klass;
        this.instanceMembers = instanceMembers;
    }

    /**
     * Looks up an instance member.
     * 
     * Note: Should only be used to look up members of class instances.
     * 
     * @param memberName The class instance members to look up.
     * @return The class member object. Null if member not found.
     */
    public ClassMember findInstanceMember(String memberName) {
        if (instanceMembers.containsKey(memberName))
            return instanceMembers.get(memberName);

        return null;
    }

    /**
     * Gets a member of this instance.
     * 
     * @param name The member's name.
     * @return The member's ClassMember object.
     */
    public ClassMember getInstanceMember(Token name) {
        ClassMember member = this.findInstanceMember(name.lexeme);
        if (member != null) {
            // Cannot access private member
            if (member.isPrivate) {
                throw new RuntimeError(
                        "Cannot access private property '" + name.lexeme + "' in class '" + this.klass.name + "'.");
            }

            // Cannot access static member form class instance
            if (member.isStatic) {
                throw new RuntimeError(name,
                        "Cannot access static property '" + name.lexeme + "' from instance of '" + klass.name + "'.");
            }

            return member;
        } else {
            throw new RuntimeError(name,
                    "Property '" + name.lexeme + "' does not exist in class '" + this.klass.name + "'.");
        }
    }

    /**
     * Modifies the value of an instance member.
     * 
     * @param name  The member's name.
     * @param value The new value to assign to the member.
     */
    public void setMemberValue(Token name, Object value) {
        ClassMember member = this.findInstanceMember(name.lexeme);

        if (member != null) {
            // Cannot set to private members
            if (member.isPrivate) {
                throw new RuntimeError(name,
                        "Cannot set to private property '" + name.lexeme + "' in class '" + klass.name + "'.");
            }

            // Cannot set to a class method
            if (member.member instanceof Stmt.Function) {
                throw new RuntimeError(name,
                        "Cannot set to method '" + name.lexeme + "' in class '" + klass.name + "'.");
            }

            // Cannot set to static members from non-static context
            if (member.isStatic) {
                throw new RuntimeError(name,
                        "Cannot set to static property '" + name.lexeme + "' from instance of '" + klass.name + "'.");
            }

            // Cannot set to final members
            if (member.isFinal) {
                throw new RuntimeError(name,
                        "Cannot set to final property '" + name.lexeme + "' in class '" + klass.name + "'.");
            }

            member.value = value;
        } else {
            throw new RuntimeError(name,
                    "Property '" + name.lexeme + "' does not exist in class '" + this.klass.name + "'.");
        }
    }

    /**
     * String representation of a class instance.
     */
    @Override
    public String toString() {
        return "<" + klass.name + " instance>";
    }
}
