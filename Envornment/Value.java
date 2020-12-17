package org.hinton_lang.Envornment;

public class Value {
    private Object value;
    public final DecType dclType;

    public Value(Object value, DecType dclType) {
        this.value = value;
        this.dclType = dclType;
    }

    /**
     * Resets the value.
     * 
     * @param value The new value.
     */
    public void setValue(Object value) {
        if (dclType != DecType.CONSTANT) {
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
    public DecType getDeclarationType() {
        return dclType;
    }
}
