#include <stdint.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>

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

/* ---- Standard library support (std modules) -------------------------- */
/* The `extern fn fen_*` declarations of the std modules are implemented    */
/* here. They are "static" on purpose: the generated prototype that         */
/* follows takes the same internal linkage. Functions marked "unchecked"    */
/* rely on the std module to validate arguments first.                      */

FEN_UNUSED static double fen_sqrt(double x) { return sqrt(x); }
FEN_UNUSED static double fen_pow(double x, double y) { return pow(x, y); }
FEN_UNUSED static double fen_floor(double x) { return floor(x); }
FEN_UNUSED static double fen_ceil(double x) { return ceil(x); }
FEN_UNUSED static double fen_round(double x) { return round(x); }
FEN_UNUSED static double fen_trunc(double x) { return trunc(x); }
FEN_UNUSED static double fen_sin(double x) { return sin(x); }
FEN_UNUSED static double fen_cos(double x) { return cos(x); }
FEN_UNUSED static double fen_tan(double x) { return tan(x); }
FEN_UNUSED static double fen_atan(double x) { return atan(x); }
FEN_UNUSED static double fen_atan2(double y, double x) { return atan2(y, x); }
FEN_UNUSED static double fen_exp(double x) { return exp(x); }
FEN_UNUSED static double fen_log(double x) { return log(x); }
FEN_UNUSED static double fen_log10(double x) { return log10(x); }

/* Three-way byte comparison: -1, 0 or 1. */
FEN_UNUSED static int32_t fen_str_cmp(const char* a, const char* b) {
    int r = strcmp(a, b);
    return r < 0 ? -1 : (r > 0 ? 1 : 0);
}

/* Index of the first `sub` in `s` at or after `from`, or -1. Unchecked:    */
/* 0 <= from <= strlen(s) is the caller's job. An empty `sub` matches at    */
/* `from`.                                                                  */
FEN_UNUSED static int32_t fen_str_find_from(const char* s, const char* sub, int32_t from) {
    const char* hit = strstr(s + from, sub);
    return hit ? (int32_t)(hit - s) : -1;
}

/* Bytes [a, b) of `s`. Unchecked: 0 <= a <= b <= strlen(s). */
FEN_UNUSED static const char* fen_str_slice(const char* s, int32_t a, int32_t b) {
    size_t len = (size_t)(b - a);
    char* r = (char*)rt_alloc(len + 1);
    memcpy(r, s + a, len);
    r[len] = 0;
    return r;
}

FEN_UNUSED static const char* fen_str_upper(const char* s) {
    size_t n = strlen(s);
    char* r = (char*)rt_alloc(n + 1);
    for (size_t i = 0; i < n; i++) {
        char c = s[i];
        r[i] = (c >= 'a' && c <= 'z') ? (char)(c - 32) : c;
    }
    r[n] = 0;
    return r;
}

FEN_UNUSED static const char* fen_str_lower(const char* s) {
    size_t n = strlen(s);
    char* r = (char*)rt_alloc(n + 1);
    for (size_t i = 0; i < n; i++) {
        char c = s[i];
        r[i] = (c >= 'A' && c <= 'Z') ? (char)(c + 32) : c;
    }
    r[n] = 0;
    return r;
}

FEN_UNUSED static const char* fen_str_reverse(const char* s) {
    size_t n = strlen(s);
    char* r = (char*)rt_alloc(n + 1);
    for (size_t i = 0; i < n; i++) r[i] = s[n - 1 - i];
    r[n] = 0;
    return r;
}

FEN_UNUSED static int fen_is_space(char c) {
    return c == ' ' || c == '\t' || c == '\n' || c == '\r' || c == '\v' || c == '\f';
}

/* Trims ASCII whitespace: mode 0 = both ends, 1 = start, 2 = end. */
FEN_UNUSED static const char* fen_str_trim(const char* s, int32_t mode) {
    size_t a = 0, b = strlen(s);
    if (mode != 2) while (a < b && fen_is_space(s[a])) a++;
    if (mode != 1) while (b > a && fen_is_space(s[b - 1])) b--;
    return fen_str_slice(s, (int32_t)a, (int32_t)b);
}

FEN_UNUSED static const char* fen_str_repeat(const char* s, int32_t n) {
    if (n < 0) rt_panic("string_repeat: negative count");
    size_t len = strlen(s);
    if (len != 0 && (size_t)n > ((size_t)-1) / 2 / len) rt_panic("string_repeat: result too large");
    char* r = (char*)rt_alloc(len * (size_t)n + 1);
    for (int32_t i = 0; i < n; i++) memcpy(r + (size_t)i * len, s, len);
    r[len * (size_t)n] = 0;
    return r;
}

/* Replaces every non-overlapping `from` in `s` by `to`. An empty `from`   */
/* returns `s` unchanged.                                                   */
FEN_UNUSED static const char* fen_str_replace(const char* s, const char* from, const char* to) {
    size_t lf = strlen(from);
    if (lf == 0) return s;
    size_t lt = strlen(to), count = 0;
    for (const char* p = strstr(s, from); p; p = strstr(p + lf, from)) count++;
    if (count == 0) return s;
    size_t ls = strlen(s);
    char* r = (char*)rt_alloc(ls - count * lf + count * lt + 1);
    char* w = r;
    const char* p = s;
    for (const char* hit = strstr(p, from); hit; hit = strstr(p, from)) {
        memcpy(w, p, (size_t)(hit - p));
        w += hit - p;
        memcpy(w, to, lt);
        w += lt;
        p = hit + lf;
    }
    strcpy(w, p);
    return r;
}

FEN_UNUSED static double fen_str_to_f64(const char* s) { return strtod(s, NULL); }

/* `x` with exactly `digits` decimals (clamped to 0..40). */
FEN_UNUSED static const char* fen_f64_fixed(double x, int32_t digits) {
    if (digits < 0) digits = 0;
    if (digits > 40) digits = 40;
    int n = snprintf(NULL, 0, "%.*f", (int)digits, x);
    char* r = (char*)rt_alloc((size_t)n + 1);
    snprintf(r, (size_t)n + 1, "%.*f", (int)digits, x);
    return r;
}
