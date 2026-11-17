# Baris perintah

```text
fenneth <file>                               Sama dengan `fenneth run <file>`
fenneth run   <file> [opsi] [-- argumen...]  Build lalu jalankan (argumen masuk ke program)
fenneth build <file> [-o out] [opsi]         Kompilasi ke executable native
fenneth emit  <file> [-o out.c]              Kompilasi ke C (default ke stdout)
fenneth check <file>                         Type-check saja
fenneth --version | --help
```

## Perintah

| Perintah | Fungsi | Butuh compiler C |
|---|---|---|
| `run` (default) | Kompilasi ke executable sementara, menjalankannya, dan meneruskan exit code-nya. | Ya |
| `build` | Menghasilkan executable native. Nama default adalah nama file tanpa ekstensi (`.exe` di Windows). | Ya |
| `emit` | Menulis C99 hasil generate ke stdout, atau ke file dari `-o`. | Tidak |
| `check` | Menjalankan lexer, parser, dan type checker. Mencetak `ok: <file>` bila berhasil. | Tidak |

```sh
fenneth hello.fen                       # sama dengan: fenneth run hello.fen
fenneth run app.fen -- arg1 arg2        # argumen setelah -- masuk ke args()
fenneth build app.fen -o app            # executable native
fenneth emit app.fen -o app.c           # hanya hasilkan C
fenneth check app.fen                   # type-check saja
```

## Opsi `build` dan `run`

| Opsi | Arti |
|---|---|
| `--cc <compiler>` | Compiler C yang dipakai. Default: `$FENNETH_CC`, lalu `gcc`, `clang`, `cc`. |
| `--link <arg>` | Argumen tambahan untuk compiler/linker C. Bisa berulang (library, `-lm`, ...). |
| `--keep-c` | Simpan file `.c` hasil generate di sebelah output. |
| `-o <file>` | Path output (`build`, `emit`). |

Compiler C dipanggil dengan `-std=c99 -O2 -fwrapv -w`. Jika Anda meng-compile hasil `emit` sendiri, tambahkan `-fwrapv` agar overflow `i32` berperilaku sama (membungkus).

## Nama file

Ekstensi boleh dihilangkan: `fenneth app` mencari `app.fen`, lalu `app.fenneth`. Hanya satu file input yang diterima di baris perintah; pakai [`import`](/id/language/modules-ffi) untuk lebih dari satu.

## Exit code dan error

- Error kompilasi mencetak setiap diagnostik sebagai `file:baris:kolom: error: pesan`, lalu `aborting: N errors`, dan keluar dengan kode `1`. Error lexer dan parser berhenti pada yang pertama; error type checker dilaporkan sekaligus.
- Program yang berjalan keluar dengan nilai yang dikembalikan `main`, atau kode dari `exit(code)`.
- Kegagalan pemeriksaan runtime (`panic`) mencetak `panic: ...` ke stderr dan keluar dengan kode `101`.

## Environment

| Variabel | Efek |
|---|---|
| `FENNETH_CC` | Compiler C default untuk `build` dan `run`. |
