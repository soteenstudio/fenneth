---
layout: home

hero:
  name: Fenneth
  text: Bahasa pemrograman bertipe statis yang kecil dan dikompilasi ke C
  tagline: Sintaks modern, type checker yang melaporkan semua error sekaligus, dan executable native lewat GCC atau Clang. Compiler ditulis dalam Rust tanpa dependensi.
  actions:
    - theme: brand
      text: Mulai
      link: /id/guide/installation
    - theme: alt
      text: Referensi bahasa
      link: /id/language/basics

features:
  - title: Dikompilasi ke C99
    details: Program Anda menjadi kode C yang mudah dibaca, lalu GCC atau Clang membuat executable native. Pakai `fenneth emit` untuk mendapatkan kode C saja.
  - title: Bertipe statis
    details: i32, f64, bool, String, array, dan struct. Tanpa konversi implisit, dan type checker melaporkan semua error dalam satu kali jalan.
  - title: Multi-file dan FFI
    details: Pecah kode dengan `import`, dan panggil fungsi C atau Rust dengan `extern fn`.
  - title: Mudah dipasang
    details: Satu perintah di Linux, macOS, Windows, dan Termux. Installer memverifikasi checksum SHA-256.
---
