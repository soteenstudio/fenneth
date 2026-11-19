/* ---- std/http: fetching URLs over HTTP(S) ------------------------------ */
/* Included only when the program uses std/http. Requests are made with the */
/* `curl` command line tool, which ships with Windows 10+, macOS and most   */
/* Linux systems; `wget` is used when curl is missing. Nothing is linked.   */
/*                                                                          */
/* curl is driven through a config file (-K), never a shell command line,   */
/* so a URL is never parsed as shell syntax.                                */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>

#ifdef _WIN32
#include <process.h>
#define fen_http_pid() ((long)_getpid())
#else
#include <unistd.h>
#define fen_http_pid() ((long)getpid())
#endif

/* Status code of the most recent request, 0 when it failed. */
FEN_UNUSED static int32_t fen_http_last_status;
/* Message of the most recent failure ("" when there was none). */
FEN_UNUSED static const char* fen_http_last_error = "";

FEN_UNUSED static int32_t fen_http_status(void) { return fen_http_last_status; }
FEN_UNUSED static const char* fen_http_error(void) { return fen_http_last_error; }

/* ---- helpers ---------------------------------------------------------- */

/* A fresh path in the temporary directory. The file is not created. */
static const char* fen_http_tmp_path(const char* tag) {
    static int counter = 0;
    const char* dir = getenv("TMPDIR");
    if (!dir || !*dir) dir = getenv("TEMP");
    if (!dir || !*dir) dir = getenv("TMP");
    if (!dir || !*dir) dir = "/tmp";
    size_t n = strlen(dir);
    bool sep = n > 0 && (dir[n - 1] == '/' || dir[n - 1] == '\\');
    char name[256];
    snprintf(name, sizeof name, "fenneth-%s-%ld-%d", tag, fen_http_pid(), ++counter);
    size_t m = strlen(name);
    char* full = (char*)rt_alloc(n + m + 2);
    memcpy(full, dir, n);
    size_t at = n;
    if (!sep) full[at++] = '/';
    memcpy(full + at, name, m + 1);
#ifdef _WIN32
    for (char* p = full; *p; p++) {
        if (*p == '/') *p = '\\';
    }
#endif
    return full;
}

/* Reads a whole file ("" when it cannot be opened). */
static const char* fen_http_read_text(const char* path) {
    FILE* f = fopen(path, "rb");
    if (!f) return "";
    size_t cap = 1024, len = 0, n;
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

static bool fen_http_write_text(const char* path, const char* text) {
    FILE* f = fopen(path, "wb");
    if (!f) return false;
    size_t n = strlen(text);
    bool ok = n == 0 || fwrite(text, 1, n, f) == n;
    if (fclose(f) != 0) ok = false;
    return ok;
}

/* Quotes a path for `system()`: sh on POSIX, cmd.exe on Windows. */
static const char* fen_http_shell_quote(const char* s) {
#ifdef _WIN32
    size_t n = strlen(s);
    char* r = (char*)rt_alloc(n + 3);
    r[0] = '"';
    memcpy(r + 1, s, n);
    r[n + 1] = '"';
    r[n + 2] = 0;
    return r;
#else
    size_t n = strlen(s);
    char* r = (char*)rt_alloc(n * 4 + 3);
    char* w = r;
    *w++ = '\'';
    for (size_t i = 0; i < n; i++) {
        if (s[i] == '\'') {
            memcpy(w, "'\\''", 4);
            w += 4;
        } else {
            *w++ = s[i];
        }
    }
    *w++ = '\'';
    *w = 0;
    return r;
#endif
}

/* Writes `key = "value"` to a curl config file. */
static void fen_http_conf(FILE* f, const char* key, const char* value) {
    fputs(key, f);
    fputs(" = \"", f);
    for (const char* p = value; p && *p; p++) {
        if (*p == '"' || *p == '\\') fputc('\\', f);
        fputc(*p, f);
    }
    fputs("\"\n", f);
}

/* A URL must not be able to break out of a shell command or a config file. */
static bool fen_http_url_ok(const char* url) {
    if (!url || !*url) return false;
    for (const char* p = url; *p; p++) {
        if (*p == '\n' || *p == '\r') return false;
    }
    return true;
}

static bool fen_http_have(const char* tool) {
#ifdef _WIN32
    char command[64];
    snprintf(command, sizeof command, "%s --version > NUL 2>&1", tool);
#else
    char command[64];
    snprintf(command, sizeof command, "%s --version > /dev/null 2>&1", tool);
#endif
    return system(command) == 0;
}

/* Cached answers; checking costs a process, so do it once. */
static int fen_http_have_curl = -1;
static int fen_http_have_wget = -1;

static bool fen_http_curl(void) {
    if (fen_http_have_curl < 0) fen_http_have_curl = fen_http_have("curl") ? 1 : 0;
    return fen_http_have_curl == 1;
}

static bool fen_http_wget(void) {
    if (fen_http_have_wget < 0) fen_http_have_wget = fen_http_have("wget") ? 1 : 0;
    return fen_http_have_wget == 1;
}

/* The status code after the last "HTTP/" line of a header dump. */
static int32_t fen_http_code_from_headers(const char* headers) {
    int32_t code = 0;
    const char* p = headers;
    while ((p = strstr(p, "HTTP/")) != NULL) {
        p += 5;
        while (*p && *p != ' ' && *p != '\r' && *p != '\n') p++; /* version */
        while (*p == ' ') p++;
        if (*p >= '0' && *p <= '9') {
            int32_t v = 0;
            while (*p >= '0' && *p <= '9') v = v * 10 + (*p++ - '0');
            code = v;
        }
    }
    return code;
}

/* Runs one request and copies the response body to `dest`.
 *
 * `method` is "GET" or "POST"; `body` is NULL for GET. Returns true when the
 * request produced a response; the code is in fen_http_last_status. */
static bool fen_http_run(const char* method, const char* url, const char* content_type,
                         const char* body, const char* dest) {
    if (!fen_http_url_ok(url)) {
        fen_http_last_status = 0;
        fen_http_last_error = "invalid URL (empty, or contains a newline)";
        return false;
    }

    const char* headers = fen_http_tmp_path("head");
    const char* errpath = fen_http_tmp_path("err");
    bool ok = false;

    if (fen_http_curl()) {
        const char* config = fen_http_tmp_path("conf");
        const char* bodypath = body ? fen_http_tmp_path("body") : NULL;
        if (bodypath && !fen_http_write_text(bodypath, body)) {
            fen_http_last_status = 0;
            fen_http_last_error = "cannot write the request body to a temporary file";
            return false;
        }
        FILE* f = fopen(config, "wb");
        if (!f) {
            fen_http_last_status = 0;
            fen_http_last_error = "cannot write a temporary curl config file";
            return false;
        }
        fen_http_conf(f, "url", url);
        fen_http_conf(f, "output", dest);
        fen_http_conf(f, "dump-header", headers);
        if (bodypath) {
            fen_http_conf(f, "request", method);
            fen_http_conf(f, "header", (content_type && *content_type)
                                            ? content_type
                                            : "application/octet-stream");
            /* `data-binary = "@file"`: curl reads the body from the file. */
            fputs("data-binary = \"@", f);
            for (const char* p = bodypath; *p; p++) {
                if (*p == '"' || *p == '\\') fputc('\\', f);
                fputc(*p, f);
            }
            fputs("\"\n", f);
        } else if (strcmp(method, "GET") != 0) {
            fen_http_conf(f, "request", method);
        }
        fclose(f);

        char command[4096];
        snprintf(command, sizeof command, "curl -sS -L --max-time 600 -K %s 2> %s",
                 fen_http_shell_quote(config), fen_http_shell_quote(errpath));
        int rc = system(command);
        fen_http_last_status = fen_http_code_from_headers(fen_http_read_text(headers));
        if (rc != 0 || fen_http_last_status == 0) {
            const char* msg = fen_http_read_text(errpath);
            fen_http_last_error = *msg ? msg : "request failed (curl)";
            fen_http_last_status = 0;
            ok = false;
        } else {
            fen_http_last_error = "";
            ok = true;
        }
        remove(config);
        remove(headers);
        remove(errpath);
        if (bodypath) remove(bodypath);
    } else if (fen_http_wget()) {
        /* wget cannot take a config file, so refuse anything a shell reads. */
        for (const char* p = url; *p; p++) {
            if (strchr("\"'`$;&|<>(){}[]*?!\\ \t", *p)) {
                fen_http_last_status = 0;
                fen_http_last_error =
                    "install curl: this URL needs it (the wget fallback only takes plain URLs)";
                return false;
            }
        }
        char command[4096];
        if (body) {
            snprintf(command, sizeof command,
                     "wget -q -O %s --header=%s --post-data=%s %s",
                     fen_http_shell_quote(dest),
                     fen_http_shell_quote((content_type && *content_type) ? content_type
                                                                        : "text/plain"),
                     fen_http_shell_quote(body), fen_http_shell_quote(url));
        } else {
            snprintf(command, sizeof command, "wget -q -O %s %s", fen_http_shell_quote(dest),
                     fen_http_shell_quote(url));
        }
        ok = system(command) == 0;
        fen_http_last_status = ok ? 200 : 0;
        fen_http_last_error = ok ? "" : "request failed (wget)";
    } else {
        fen_http_last_status = 0;
        fen_http_last_error = "neither curl nor wget was found on PATH";
        return false;
    }

    if (ok) {
        /* A 4xx/5xx answer is still a response; the caller decides. */
        if (fen_http_last_status >= 400) ok = true;
    }
    return ok;
}

/* ---- the functions std/http declares ---------------------------------- */

/* GET `url`; the response body ("" when the request failed). */
FEN_UNUSED static const char* fen_http_get(const char* url) {
    fen_http_last_status = 0;
    fen_http_last_error = "";
    const char* body = fen_http_tmp_path("resp");
    if (!fen_http_run("GET", url, NULL, NULL, body)) return "";
    return fen_http_read_text(body);
}

/* POST `body` to `url`; the response body. */
FEN_UNUSED static const char* fen_http_post(const char* url, const char* content_type,
                                            const char* body) {
    fen_http_last_status = 0;
    fen_http_last_error = "";
    const char* resp = fen_http_tmp_path("resp");
    if (!fen_http_run("POST", url, content_type, body ? body : "", resp)) return "";
    return fen_http_read_text(resp);
}

/* Downloads `url` into the file at `path`. True when it worked. */
FEN_UNUSED static bool fen_http_download(const char* url, const char* path) {
    fen_http_last_status = 0;
    fen_http_last_error = "";
    return fen_http_run("GET", url, NULL, NULL, path);
}

/* Percent-encodes a string so it can go inside a URL. */
FEN_UNUSED static const char* fen_url_encode(const char* s) {
    static const char* hex = "0123456789ABCDEF";
    size_t n = strlen(s);
    char* r = (char*)rt_alloc(n * 3 + 1);
    size_t w = 0;
    for (size_t i = 0; i < n; i++) {
        unsigned char c = (unsigned char)s[i];
        bool plain = (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z') ||
                     (c >= '0' && c <= '9') || c == '-' || c == '_' || c == '.' || c == '~';
        if (plain) {
            r[w++] = (char)c;
        } else {
            r[w++] = '%';
            r[w++] = hex[c >> 4];
            r[w++] = hex[c & 15];
        }
    }
    r[w] = 0;
    return r;
}

/* Decodes %XX escapes and '+' (form encoding). Bad escapes stay as they are. */
FEN_UNUSED static const char* fen_url_decode(const char* s) {
    size_t n = strlen(s);
    char* r = (char*)rt_alloc(n + 1);
    size_t w = 0;
    for (size_t i = 0; i < n; i++) {
        if (s[i] == '%' && i + 2 < n) {
            int hi = s[i + 1], lo = s[i + 2];
            int a = hi >= '0' && hi <= '9' ? hi - '0'
                    : hi >= 'A' && hi <= 'F' ? hi - 'A' + 10
                    : hi >= 'a' && hi <= 'f' ? hi - 'a' + 10
                                             : -1;
            int b = lo >= '0' && lo <= '9' ? lo - '0'
                    : lo >= 'A' && lo <= 'F' ? lo - 'A' + 10
                    : lo >= 'a' && lo <= 'f' ? lo - 'a' + 10
                                             : -1;
            if (a >= 0 && b >= 0) {
                r[w++] = (char)(a * 16 + b);
                i += 2;
                continue;
            }
        }
        r[w++] = s[i] == '+' ? ' ' : s[i];
    }
    r[w] = 0;
    return r;
}
