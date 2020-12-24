package org.hinton_lang.Interpreter.HintonDictionary;

import java.util.List;
import java.util.HashMap;

import org.hinton_lang.Errors.RuntimeError;
import org.hinton_lang.Interpreter.NativeType;
import org.hinton_lang.Parser.AST.Expr.KeyValPair;
import org.hinton_lang.Tokens.Token;
import org.hinton_lang.Interpreter.Interpreter;

/**
 * Represents a Hinton Dictionary in the Hinton language.
 */
public class HintonDictionary implements NativeType {
    private final HashMap<String, Object> members = new HashMap<>();
    private final HashMap<String, Object> methods = new HashMap<>();
    private final Interpreter interpreter;

    /**
     * Wraps a Java HashMap object so that it can be interpreted as a Hinton
     * Dictionary.
     * 
     * @param interpreter THe interpreter instance.
     * @param members     The initial key-value pairs for the dictionary.
     */
    public HintonDictionary(Interpreter interpreter, List<KeyValPair> members) {
        this.interpreter = interpreter;

        // The methods for dictionary objects in the Hinton language
        methods.put("size", this.members.size());

        // Adds the initial dictionary members
        for (KeyValPair member : members) {
            if (methods.containsKey(member.key.lexeme)) {
                throw new RuntimeError(member.key,
                        "Cannot redeclare built-in dictionary member '" + member.key.lexeme + "'.");
            } else if (this.members.containsKey(member.key.lexeme)) {
                throw new RuntimeError(member.key, "Cannot redeclare dictionary member '" + member.key.lexeme + "'.");
            } else {
                this.members.put(member.key.lexeme, interpreter.evaluate(member.val));
            }
        }
    }

    /**
     * Gets the given Hinton Dict property.
     * 
     * @param prop The property to be accessed.
     * @return The property's value.
     */
    public Object getProperty(Token prop) {
        // We first check if the requested property is a built-in member.
        if (methods.containsKey(prop.lexeme)) {
            return methods.get(prop.lexeme);
        }

        // We then check if it is inside the keyset for the dictionary
        if (members.containsKey(prop.lexeme)) {
            return members.get(prop.lexeme);
        }

        throw new RuntimeError(prop, "Property '" + prop.lexeme + "' does not exist in Dictionary.");
    }

    /**
     * Returns the raw HashMap in this wrapper.
     * 
     * @return The raw HashMap.
     */
    public Object getRaw() {
        return this.members;
    }

    /**
     * Return the Hinton type name for the object.
     */
    public String typeName() {
        return "Dict";
    }

    /**
     * String representation of a Hinton Dict.
     */
    @Override
    public String toString() {
        StringBuilder str = new StringBuilder("{ ");

        Object[] keys = this.members.keySet().toArray();
        Object[] values = this.members.values().toArray();

        for (int i = 0; i < this.members.size(); i++) {
            Object key = keys[i];
            Object value = values[i];

            str.append(key);
            str.append(": ");

            if (value instanceof NativeType) {
                str.append(((NativeType) value).formattedStr());
            } else {
                str.append(value);
            }

            // Adds a comma separator
            if (i < this.members.size() - 1)
                str.append(", ");
        }

        str.append(" }");

        return str.toString();
    }

    /**
     * Formatted string representation of a Hinton Dict.
     */
    @Override
    public String formattedStr() {
        return this.toString();
    }
}
