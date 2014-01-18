RUSTC = rustc
BUILDDIR = build
RUSTFLAGS = -O

PHF_LIB = lib.rs
PHF = $(BUILDDIR)/$(shell $(RUSTC) --crate-file-name $(PHF_LIB))
PHF_TEST_MAIN = test.rs
PHF_TEST = $(BUILDDIR)/$(shell $(RUSTC) --crate-file-name $(PHF_TEST_MAIN))

all: $(PHF)

-include $(BUILDDIR)/phf.d
-include $(BUILDDIR)/phf_test.d

$(BUILDDIR):
	mkdir -p $@

$(PHF): $(PHF_LIB) | $(BUILDDIR)
	$(RUSTC) $(RUSTFLAGS) --dep-info $(BUILDDIR)/phf.d \
		--out-dir $(BUILDDIR) $<

$(PHF_TEST): $(PHF_TEST_MAIN) $(PHF) | $(BUILDDIR)
	$(RUSTC) --test $(RUSTFLAGS) -L $(BUILDDIR) \
		--dep-info $(BUILDDIR)/phf_test.d --out-dir $(BUILDDIR) $<

check: $(PHF_TEST)
	$(PHF_TEST)

clean:
	rm -rf $(BUILDDIR)

.PHONY: all clean check
