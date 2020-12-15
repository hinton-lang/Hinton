package org.hinton_lang.Envornment;

public class Value {
    private Object value;
    public final Symbol signature;

    public Value(Object value, Symbol signature) {
        this.value = value;
        this.signature = signature;
    }

    /**
     * Resets the value.
     * 
     * @param value The new value.
     */
    public void setValue(Object value) {
        if (signature != Symbol.CONSTANT) {
            this.value = value;
        }
    }

    /**
     * Gets the value.
     * 
     * @return The stored value.
     */
    public Object getValue() {
        return this.value;
    }

    /**
     * Gets the signature type.
     * 
     * @return The signature type.
     */
    public Symbol getSignature() {
        return signature;
    }
}
