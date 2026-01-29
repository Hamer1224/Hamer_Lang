# **H@mer (v0.1.0) 🔨**

H@mer is a high-performance, entropy-driven systems programming language built specifically for ARM64 (AArch64) Linux.
​It is designed as a "Hybrid Language," allowing seamless transitions between structured high-level logic and raw hardware mnemonics.

## ​Key Features
​. Entropy-Driven Logic: Native probabilistic branching using the hardware cycle counter (?<%10>).

​. Kernel-Level I/O: No standard library. Every print and rest command is a direct Linux syscall.

. ​Dynamic Memory: Custom heap allocation using mmap syscalls.

​. Inline Assembly: First-class support for raw ARM64 assembly blocks with @asm is ... done.

## Example Syntax
```h@mer
GET math
local math = new MathLib

if ?<%30> is
    print "You hit the 30% chance jackpot!"
done
```

## Compilation Pipeline
​H@mer compiles to ARM64 and Intel assembly, which is then handled by the GNU Assembler (as) and Linker (ld).

## Project Structure
. ​src/lexer.rs: Tokenizes the source code.

. ​src/parser.rs: Builds the Abstract Syntax Tree (AST).

. ​src/generator.rs: Emits optimized ARM64 Assembly.

. ​src/math.hmr: The hardware entropy library.
