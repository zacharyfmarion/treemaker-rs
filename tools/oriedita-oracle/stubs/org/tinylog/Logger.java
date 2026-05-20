package org.tinylog;

public final class Logger {
    private Logger() {
    }

    public static void info(String message) {
    }

    public static void error(Throwable throwable, String message) {
        throwable.printStackTrace(System.err);
    }
}
