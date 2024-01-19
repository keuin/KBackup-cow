package com.keuin.kbackupfabric.util.cow;


public final class FileCowCopier {

    public static native void init();

    public static native void copy(String dst, String src);

    public static native String getVersion();

    public static void main(String[] args) {
        System.loadLibrary("kbackup_cow");
        FileCowCopier.init();
        System.out.println("kbackup-cow version: " + FileCowCopier.getVersion());
        FileCowCopier.copy("b", "a");
    }

}
