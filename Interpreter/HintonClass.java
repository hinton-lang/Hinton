package org.hinton_lang.Interpreter;

import java.util.HashMap;
import java.util.List;

import org.hinton_lang.Errors.RuntimeError;
import org.hinton_lang.Tokens.Token;

/**
 * Represents a class declaration.
 */
public class HintonClass implements HintonInstantiable {
    public final String name;
    private final HashMap<String, ClassMember> staticMembers;
    private final HashMap<String, ClassMember> instanceMembers;

    /**
     * Represents a class declaration.
     * 
     * @param name    The name of the class.
     * @param members The members of the class.
     */
    public HintonClass(String name, HashMap<String, ClassMember> staticMembers,
            HashMap<String, ClassMember> instanceMembers) {
        this.name = name;
        this.staticMembers = staticMembers;
        this.instanceMembers = instanceMembers;
    }

    /**
     * Loops up a static member.
     * 
     * Note: Should only be used to retrieve a static member from the class.
     * 
     * @param name The member's name.
     * @return The class member object.
     */
    public ClassMember getStaticMember(Token name) throws RuntimeError {
        if (staticMembers.containsKey(name.lexeme)) {
            if (staticMembers.get(name.lexeme).isStatic) {
                return staticMembers.get(name.lexeme);
            } else {
                throw new RuntimeError(name, "Cannot access property '" + name.lexeme + "' from outside instance.");
            }
        }

        throw new RuntimeError(name,
                "Static property '" + name.lexeme + "' does not exist in class '" + this.name + "'.");
    }

    /**
     * Instantiates the class. (Equivalent to `call()` in HintonFunction).
     */
    @Override
    public Object instantiate(Interpreter interpreter, List<Object> arguments) {
        // Clones the member of the class into the new instance.
        HashMap<String, ClassMember> clonedMembers = new HashMap<String, ClassMember>();
        for (String name : instanceMembers.keySet()) {
            try {
                clonedMembers.put(name, instanceMembers.get(name).clone());
            } catch (CloneNotSupportedException e) {
                System.out.println("Unexpected error. Cannot create instance of class " + this.name + ".");
            }
        }
        // Creates a new instance of the class with the cloned class members.
        HintonInstance instance = new HintonInstance(this, clonedMembers);
        return instance;
    }

    /**
     * Class arguments.
     */
    @Override
    public int arity() {
        return 0;
    }

    /**
     * Class string representation.
     */
    @Override
    public String toString() {
        return "<Class " + this.name + ">";
    }
}