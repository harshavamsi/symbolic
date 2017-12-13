extern crate cc;

fn main() {
    cc::Build::new()
        .warnings(false)
        .file("third_party/breakpad/third_party/libdisasm/ia32_implicit.c")
        .file("third_party/breakpad/third_party/libdisasm/ia32_insn.c")
        .file("third_party/breakpad/third_party/libdisasm/ia32_invariant.c")
        .file("third_party/breakpad/third_party/libdisasm/ia32_modrm.c")
        .file("third_party/breakpad/third_party/libdisasm/ia32_opcode_tables.c")
        .file("third_party/breakpad/third_party/libdisasm/ia32_operand.c")
        .file("third_party/breakpad/third_party/libdisasm/ia32_reg.c")
        .file("third_party/breakpad/third_party/libdisasm/ia32_settings.c")
        .file("third_party/breakpad/third_party/libdisasm/x86_disasm.c")
        .file("third_party/breakpad/third_party/libdisasm/x86_format.c")
        .file("third_party/breakpad/third_party/libdisasm/x86_imm.c")
        .file("third_party/breakpad/third_party/libdisasm/x86_insn.c")
        .file("third_party/breakpad/third_party/libdisasm/x86_misc.c")
        .file("third_party/breakpad/third_party/libdisasm/x86_operand_list.c")
        .compile("disasm");

    cc::Build::new()
        .cpp(true)
        .warnings(false)
        .flag("-std=c++11")
        .include(".")
        .include("third_party/breakpad")
        .define("BPLOG_MINIMUM_SEVERITY", "SEVERITY_ERROR")

        // Processor
        .file("third_party/breakpad/processor/basic_code_modules.cc")
        .file("third_party/breakpad/processor/basic_source_line_resolver.cc")
        .file("third_party/breakpad/processor/call_stack.cc")
        .file("third_party/breakpad/processor/cfi_frame_info.cc")
        .file("third_party/breakpad/processor/disassembler_x86.cc")
        .file("third_party/breakpad/processor/dump_context.cc")
        .file("third_party/breakpad/processor/dump_object.cc")
        .file("third_party/breakpad/processor/logging.cc")
        .file("third_party/breakpad/processor/pathname_stripper.cc")
        .file("third_party/breakpad/processor/process_state.cc")
        .file("third_party/breakpad/processor/proc_maps_linux.cc")
        .file("third_party/breakpad/processor/simple_symbol_supplier.cc")
        .file("third_party/breakpad/processor/source_line_resolver_base.cc")
        .file("third_party/breakpad/processor/stack_frame_cpu.cc")
        .file("third_party/breakpad/processor/stack_frame_symbolizer.cc")
        .file("third_party/breakpad/processor/stackwalker.cc")
        .file("third_party/breakpad/processor/stackwalker_amd64.cc")
        .file("third_party/breakpad/processor/stackwalker_arm.cc")
        .file("third_party/breakpad/processor/stackwalker_arm64.cc")
        .file("third_party/breakpad/processor/stackwalker_mips.cc")
        .file("third_party/breakpad/processor/stackwalker_ppc.cc")
        .file("third_party/breakpad/processor/stackwalker_ppc64.cc")
        .file("third_party/breakpad/processor/stackwalker_sparc.cc")
        .file("third_party/breakpad/processor/stackwalker_x86.cc")
        .file("third_party/breakpad/processor/tokenize.cc")

        // Minidump
        .file("third_party/breakpad/processor/exploitability.cc")
        .file("third_party/breakpad/processor/exploitability_linux.cc")
        .file("third_party/breakpad/processor/exploitability_win.cc")
        .file("third_party/breakpad/processor/minidump.cc")
        .file("third_party/breakpad/processor/minidump_processor.cc")
        .file("third_party/breakpad/processor/symbolic_constants_win.cc")

        // Symbolic bindings
        .file("cpp/c_string.cpp")
        .file("cpp/data_structures.cpp")
        .file("cpp/mmap_symbol_supplier.cpp")
        .file("cpp/processor.cpp")
        .compile("breakpad");
}
