package org.example;

import java.lang.reflect.InvocationHandler;
import java.lang.reflect.Method;
import java.util.function.Consumer;

public class ConsumerCallback implements InvocationHandler, Consumer<Object> {

    private final long nativeCallbackPtr;

    public ConsumerCallback() {
        System.out.println("Default constructor called");
        this.nativeCallbackPtr = 0;
    }

    public ConsumerCallback(long nativeCallbackPtr) {
        System.out.println("Constructor called with: " + nativeCallbackPtr);
        this.nativeCallbackPtr = nativeCallbackPtr;
    }

    @Override
    public void accept(Object event) {
        //System.out.println("accept called with event: " + event.getClass().getName());
        //System.out.println("Event details: " + event.toString());
        invokeNativeCallback(nativeCallbackPtr, event);
        //System.out.println("Native callback completed for event: " + event.getClass().getName());
    }

    @Override
    public Object invoke(Object proxy, Method method, Object[] args) throws Throwable {
        System.out.println("invoke called for method: " + method.getName());
        System.out.println("proxy class: " + proxy.getClass().getName());
        System.out.println("method declaring class: " + method.getDeclaringClass().getName());
        if (args != null) {
            System.out.println("args length: " + args.length);
            for (int i = 0; i < args.length; i++) {
                if (args[i] != null) {
                    System.out.println("arg " + i + " class: " + args[i].getClass().getName());
                    System.out.println("arg " + i + " toString: " + args[i].toString());
                } else {
                    System.out.println("arg " + i + ": null");
                }
            }
        }

        if ("accept".equals(method.getName()) && args != null && args.length > 0) {
            System.out.println("Calling native callback with ptr: " + nativeCallbackPtr);
            System.out.println("Event class: " + args[0].getClass().getName());
            System.out.println("Event details: " + args[0].toString());
            invokeNativeCallback(nativeCallbackPtr, args[0]);
            System.out.println("Native callback completed");
        }
        return null;
    }

    private native void invokeNativeCallback(long ptr, Object event);
} 