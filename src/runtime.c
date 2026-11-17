#include <stdint.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#if defined(__GNUC__)
#define FEN_UNUSED __attribute__((unused))
#else
#define FEN_UNUSED
#endif

/* ---- Fenneth runtime -------------------------------------------------- */
/* Strings are immutable NUL-terminated byte strings. Heap strings and     */
/* arrays are never freed (the program is expected to be short-lived).     */

FEN_UNUSED static int g_argc;
FEN_UNUSED static char** g_argv;

FEN_UNUSED static void rt_panic(const char* msg) {
    fflush(stdout);
    fprintf(stderr, "panic: %s\n", msg);
    exit(101);
}

FEN_UNUSED static void rt_index_panic(int32_t i, int32_t len) {
    fflush(stdout);
    fprintf(stderr, "panic: index %d out of bounds (length %d)\n", (int)i, (int)len);
    exit(101);
}

FEN_UNUSED static void* rt_alloc(size_t n) {
    void* p = malloc(n ? n : 1);
    if (!p) rt_panic("out of memory");
    return p;
}

FEN_UNUSED static void* rt_realloc(void* p, size_t n) {
    void* q = realloc(p, n ? n : 1);
    if (!q) rt_panic("out of memory");
    return q;
}

FEN_UNUSED static int32_t rt_div(int32_t a, int32_t b) {
    if (b == 0) rt_panic("division by zero");
    if (b == -1) return (int32_t)(0u - (uint32_t)a);
    return a / b;
}

FEN_UNUSED static int32_t rt_rem(int32_t a, int32_t b) {
    if (b == 0) rt_panic("remainder by zero");
    if (b == -1) return 0;
    return a % b;
}

FEN_UNUSED static const char* rt_dup(const char* s) {
    size_t n = strlen(s);
    char* r = (char*)rt_alloc(n + 1);
    memcpy(r, s, n + 1);
    return r;
}

FEN_UNUSED static const char* rt_concat(const char* a, const char* b) {
    size_t la = strlen(a), lb = strlen(b);
    char* r = (char*)rt_alloc(la + lb + 1);
    memcpy(r, a, la);
    memcpy(r + la, b, lb + 1);
    return r;
}

FEN_UNUSED static const char* rt_i32_str(int32_t v) {
    char tmp[16];
    snprintf(tmp, sizeof tmp, "%d", (int)v);
    return rt_dup(tmp);
}

FEN_UNUSED static const char* rt_f64_str(double v) {
    char tmp[64];
    snprintf(tmp, sizeof tmp, "%g", v);
    return rt_dup(tmp);
}

FEN_UNUSED static const char* rt_bool_str(bool v) {
    return v ? "true" : "false";
}

FEN_UNUSED static int32_t rt_str_at(const char* s, int32_t i) {
    size_t n = strlen(s);
    if (i < 0 || (size_t)i >= n) rt_index_panic(i, (int32_t)n);
    return (int32_t)(unsigned char)s[i];
}

FEN_UNUSED static const char* rt_substr(const char* s, int32_t a, int32_t b) {
    size_t n = strlen(s);
    if (a < 0 || b < a || (size_t)b > n) rt_panic("substr range out of bounds");
    size_t len = (size_t)(b - a);
    char* r = (char*)rt_alloc(len + 1);
    memcpy(r, s + a, len);
    r[len] = 0;
    return r;
}

FEN_UNUSED static const char* rt_char_str(int32_t c) {
    if (c < 0 || c > 255) rt_panic("char_str: code out of range 0..255");
    char* r = (char*)rt_alloc(2);
    r[0] = (char)c;
    r[1] = 0;
    return r;
}

FEN_UNUSED static int32_t rt_str_to_i32(const char* s) {
    return (int32_t)strtol(s, NULL, 10);
}

FEN_UNUSED static const char* rt_read_file(const char* path) {
    FILE* f = fopen(path, "rb");
    if (!f) return "";
    size_t cap = 4096, len = 0, n;
    char* buf = (char*)rt_alloc(cap);
    while ((n = fread(buf + len, 1, cap - len - 1, f)) > 0) {
        len += n;
        if (cap - len < 2) {
            cap *= 2;
            buf = (char*)rt_realloc(buf, cap);
        }
    }
    fclose(f);
    buf[len] = 0;
    return buf;
}

FEN_UNUSED static bool rt_write_file(const char* path, const char* content) {
    FILE* f = fopen(path, "wb");
    if (!f) return false;
    size_t n = strlen(content);
    bool ok = fwrite(content, 1, n, f) == n;
    if (fclose(f) != 0) ok = false;
    return ok;
}

FEN_UNUSED static bool rt_file_exists(const char* path) {
    FILE* f = fopen(path, "rb");
    if (!f) return false;
    fclose(f);
    return true;
}

FEN_UNUSED static const char* rt_read_line(void) {
    size_t cap = 128, len = 0;
    char* buf = (char*)rt_alloc(cap);
    int c;
    while ((c = fgetc(stdin)) != EOF) {
        if (c == '\n') break;
        if (len + 1 >= cap) {
            cap *= 2;
            buf = (char*)rt_realloc(buf, cap);
        }
        buf[len++] = (char)c;
    }
    if (len > 0 && buf[len - 1] == '\r') len--;
    buf[len] = 0;
    return buf;
}
