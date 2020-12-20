package org.hinton_lang.Interpreter;

public class HintonInstance {
    private HintonClass klass;

    public HintonInstance(HintonClass klass) {
        this.klass = klass;
    }

    @Override
    public String toString() {
        return "<" + klass.name + " instance>";
    }
}
