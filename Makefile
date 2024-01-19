JAVAC = javac

HEADER_DIR = src/header
JAVA_SRC_DIR = src/java

JAVA_SRC_FILE = $(JAVA_SRC_DIR)/FileCowCopier.java
JNI_HEADER = $(HEADER_DIR)/com_keuin_kbackupfabric_util_cow_FileCowCopier.h

all: $(JNI_HEADER)

$(JNI_HEADER): $(JAVA_SRC_FILE)
	$(JAVAC) -h $(HEADER_DIR) $(JAVA_SRC_FILE)

clean:
	rm -rf $(HEADER_DIR)

.PHONY: all clean
