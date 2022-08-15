INTERFACES=${OUT}/kata/components

${INTERFACES}/MemoryManagerBindings.h: src/lib.rs cbindgen.toml
	mkdir -p ${INTERFACES}
	cbindgen -c cbindgen.toml src/lib.rs -o $@
