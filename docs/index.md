---
layout: home

hero:
  name: Fenneth
  text: A small, statically typed language that compiles to C
  tagline: Modern syntax, a type checker that reports every error at once, and native executables through GCC or Clang. The compiler is written in Rust with zero dependencies.
  actions:
    - theme: brand
      text: Get started
      link: /guide/installation
    - theme: alt
      text: Language reference
      link: /language/basics

features:
  - title: Compiles to C99
    details: Your program becomes readable C, then GCC or Clang turns it into a native executable. Use `fenneth emit` to get just the C.
  - title: Statically typed
    details: i32, f64, bool, String, arrays and structs. No implicit conversions, no surprises, and the type checker reports all errors in one pass.
  - title: Multi-file and FFI
    details: Split code with `import`, and call C or Rust functions with `extern fn`.
  - title: Easy to install
    details: One command on Linux, macOS, Windows and Termux. Installers verify SHA-256 checksums.
---
