package org.hinton_lang.Interpreter;

import java.util.List;
import java.util.Map;

public class HintonClass implements HintonInstantiable {
    public final String name;

    public HintonClass(String name) {
        this.name = name;
    }

    @Override
    public Object instantiate(Interpreter interpreter, List<Object> arguments) {
        HintonInstance instance = new HintonInstance(this);
        return instance;
    }

    @Override
    public int arity() {
        return 0;
    }

    @Override
    public String toString() {
        return "<Class " + this.name + ">";
    }
}