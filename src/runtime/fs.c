/* ---- std/fs: files and directories ----------------------------------- */
/* Included in the generated C only when the program uses std/fs. These are */
/* plain C library calls (POSIX on Linux/macOS/Termux, the MinGW/ucrt        */
/* equivalents on Windows), so no extra library has to be linked.           */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>
#include <sys/stat.h>
#include <dirent.h>

#ifdef _WIN32
#include <direct.h>
#include <process.h>
#define fen_fs_pid() ((long)_getpid())
#define fen_is_sep(c) ((c) == '/' || (c) == '\\')
#else
#include <unistd.h>
#define fen_fs_pid() ((long)getpid())
#define fen_is_sep(c) ((c) == '/')
#endif

FEN_UNUSED static bool fen_fs_exists(const char* path) {
    struct stat st;
    return stat(path, &st) == 0;
}

FEN_UNUSED static bool fen_fs_is_dir(const char* path) {
    struct stat st;
    return stat(path, &st) == 0 && S_ISDIR(st.st_mode);
}

FEN_UNUSED static bool fen_fs_is_file(const char* path) {
    struct stat st;
    return stat(path, &st) == 0 && S_ISREG(st.st_mode);
}

/* Size in bytes, or -1 when the path is not a readable regular file. */
FEN_UNUSED static int32_t fen_fs_size(const char* path) {
    struct stat st;
    if (stat(path, &st) != 0 || !S_ISREG(st.st_mode)) return -1;
    if (st.st_size > (long)0x7FFFFFFF) return 0x7FFFFFFF;
    return (int32_t)st.st_size;
}

FEN_UNUSED static bool fen_fs_mkdir(const char* path) {
#ifdef _WIN32
    return _mkdir(path) == 0 || fen_fs_is_dir(path);
#else
    return mkdir(path, 0777) == 0 || fen_fs_is_dir(path);
#endif
}

FEN_UNUSED static bool fen_fs_mkdir_all(const char* path) {
    char* tmp = (char*)rt_dup(path);
    size_t n = strlen(tmp);
    for (size_t i = 1; i < n; i++) {
        if (!fen_is_sep(tmp[i])) continue;
#ifdef _WIN32
        if (i == 2 && tmp[1] == ':') continue; /* "C:\" already exists */
#endif
        char saved = tmp[i];
        tmp[i] = 0;
        if (tmp[0] && !fen_fs_is_dir(tmp)) fen_fs_mkdir(tmp);
        tmp[i] = saved;
    }
    return fen_fs_mkdir(tmp);
}

FEN_UNUSED static bool fen_fs_remove(const char* path) {
    return remove(path) == 0;
}

FEN_UNUSED static bool fen_fs_remove_dir(const char* path) {
#ifdef _WIN32
    return _rmdir(path) == 0;
#else
    return rmdir(path) == 0;
#endif
}

FEN_UNUSED static bool fen_fs_rename(const char* from, const char* to) {
#ifdef _WIN32
    remove(to); /* rename() does not replace an existing file there */
#endif
    return rename(from, to) == 0;
}

FEN_UNUSED static bool fen_fs_copy(const char* from, const char* to) {
    FILE* in = fopen(from, "rb");
    if (!in) return false;
    FILE* out = fopen(to, "wb");
    if (!out) {
        fclose(in);
        return false;
    }
    char buf[8192];
    size_t n;
    bool ok = true;
    while ((n = fread(buf, 1, sizeof buf, in)) > 0) {
        if (fwrite(buf, 1, n, out) != n) {
            ok = false;
            break;
        }
    }
    fclose(in);
    if (fclose(out) != 0) ok = false;
    return ok;
}

FEN_UNUSED static bool fen_fs_append(const char* path, const char* text) {
    FILE* f = fopen(path, "ab");
    if (!f) return false;
    size_t n = strlen(text);
    bool ok = n == 0 || fwrite(text, 1, n, f) == n;
    if (fclose(f) != 0) ok = false;
    return ok;
}

/* Entry names of a directory, joined by "\n" ("" when it cannot be read). */
FEN_UNUSED static const char* fen_fs_list(const char* path) {
    DIR* d = opendir(path);
    if (!d) return "";
    size_t cap = 256, len = 0;
    char* buf = (char*)rt_alloc(cap);
    buf[0] = 0;
    struct dirent* e;
    while ((e = readdir(d)) != NULL) {
        const char* name = e->d_name;
        if (strcmp(name, ".") == 0 || strcmp(name, "..") == 0) continue;
        size_t n = strlen(name);
        while (len + n + 2 > cap) cap *= 2;
        buf = (char*)rt_realloc(buf, cap);
        if (len) buf[len++] = '\n';
        memcpy(buf + len, name, n);
        len += n;
        buf[len] = 0;
    }
    closedir(d);
    return buf;
}

FEN_UNUSED static const char* fen_fs_cwd(void) {
    char buf[4096];
    if (!getcwd(buf, sizeof buf)) return "";
    return rt_dup(buf);
}

FEN_UNUSED static const char* fen_fs_temp_dir(void) {
    const char* d = getenv("TMPDIR");
    if (!d || !*d) d = getenv("TEMP");
    if (!d || !*d) d = getenv("TMP");
    if (!d || !*d) d = "/tmp";
    return rt_dup(d);
}

FEN_UNUSED static const char* fen_fs_separator(void) {
#ifdef _WIN32
    return "\\";
#else
    return "/";
#endif
}

FEN_UNUSED static const char* fen_fs_join(const char* a, const char* b) {
    if (!a || !*a) return rt_dup(b ? b : "");
    if (!b || !*b) return rt_dup(a);
    if (fen_is_sep(b[0])) return rt_dup(b);
#ifdef _WIN32
    if (isalpha((unsigned char)b[0]) && b[1] == ':') return rt_dup(b);
#endif
    size_t la = strlen(a);
    bool need = !fen_is_sep(a[la - 1]);
    const char* sep = fen_fs_separator();
    size_t lb = strlen(b), ls = strlen(sep);
    char* r = (char*)rt_alloc(la + (need ? ls : 0) + lb + 1);
    memcpy(r, a, la);
    size_t at = la;
    if (need) {
        memcpy(r + at, sep, ls);
        at += ls;
    }
    memcpy(r + at, b, lb + 1);
    return r;
}

/* Last component of a path ("" for a path that ends in a separator). */
FEN_UNUSED static const char* fen_fs_basename(const char* path) {
    size_t n = strlen(path);
    while (n > 0 && fen_is_sep(path[n - 1])) n--;
    size_t i = n;
    while (i > 0 && !fen_is_sep(path[i - 1])) i--;
    return fen_str_slice(path, (int32_t)i, (int32_t)n);
}

/* Everything but the last component; "." when there is none. */
FEN_UNUSED static const char* fen_fs_dirname(const char* path) {
    size_t n = strlen(path);
    while (n > 0 && fen_is_sep(path[n - 1])) n--;
    if (n == 0) return fen_is_sep(path[0]) ? "/" : ".";
    size_t i = n;
    while (i > 0 && !fen_is_sep(path[i - 1])) i--;
    if (i == 0) return ".";
    size_t end = i;
    while (end > 1 && fen_is_sep(path[end - 1])) end--;
    if (end == 1 && fen_is_sep(path[0])) return "/";
#ifdef _WIN32
    if (end == 2 && path[1] == ':') end = 3; /* keep "C:\" */
#endif
    return fen_str_slice(path, 0, (int32_t)end);
}

/* Text after the last dot of the file name ("" when there is none). */
FEN_UNUSED static const char* fen_fs_extension(const char* path) {
    size_t n = strlen(path);
    while (n > 0 && fen_is_sep(path[n - 1])) n--;
    size_t i = n;
    while (i > 0 && !fen_is_sep(path[i - 1])) i--;
    for (size_t k = n; k > i; k--) {
        if (path[k - 1] == '.') {
            if (k - 1 == i) return ""; /* ".bashrc" has no extension */
            return fen_str_slice(path, (int32_t)k, (int32_t)n);
        }
    }
    return "";
}

/* A new, unused path inside the temporary directory. Not created. */
FEN_UNUSED static const char* fen_fs_temp_path(const char* prefix) {
    static int counter = 0;
    char name[128];
    size_t w = 0;
    const char* p = prefix;
    for (; p && *p && w < 48; p++) {
        unsigned char c = (unsigned char)*p;
        if (isalnum(c) || c == '-' || c == '_' || c == '.') name[w++] = (char)c;
        else name[w++] = '_';
    }
    if (w == 0) {
        const char* d = "fenneth";
        while (*d && w < 48) name[w++] = *d++;
    }
    name[w] = 0;
    char full[160];
    snprintf(full, sizeof full, "%s-%ld-%d.tmp", name, fen_fs_pid(), ++counter);
    return fen_fs_join(fen_fs_temp_dir(), full);
}
