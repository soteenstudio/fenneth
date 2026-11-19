/* ---- std/zip: reading and writing .zip archives ------------------------ */
/* Included only when the program uses std/zip. Everything is plain C: the  */
/* reader handles stored (method 0) and deflated (method 8) entries, so it  */
/* opens any normal .zip; the writer stores entries uncompressed (a valid,  */
/* standard archive without the DEFLATE compressor). CRC-32 is verified.    */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <stdbool.h>
#include <time.h>
#include <sys/stat.h>
#include <sys/types.h>

#ifdef _WIN32
#include <direct.h>
#define fen_zip_sep(c) ((c) == '/' || (c) == '\\')
#define fen_zip_mkdir1(p) _mkdir(p)
#else
#define fen_zip_sep(c) ((c) == '/')
#define fen_zip_mkdir1(p) mkdir(p, 0777)
#endif

FEN_UNUSED static const char* fen_zip_last_error = "";
FEN_UNUSED static const char* fen_zip_error(void) { return fen_zip_last_error; }

/* ---- small helpers ---------------------------------------------------- */

static uint32_t fen_zip_crc32(const unsigned char* data, size_t len) {
    static uint32_t table[256];
    static int ready = 0;
    if (!ready) {
        for (uint32_t i = 0; i < 256; i++) {
            uint32_t c = i;
            for (int k = 0; k < 8; k++) c = (c & 1) ? 0xEDB88320u ^ (c >> 1) : c >> 1;
            table[i] = c;
        }
        ready = 1;
    }
    uint32_t c = 0xFFFFFFFFu;
    for (size_t i = 0; i < len; i++) c = table[(c ^ data[i]) & 0xFF] ^ (c >> 8);
    return c ^ 0xFFFFFFFFu;
}

static uint32_t fen_zip_u32(const unsigned char* p) {
    return (uint32_t)p[0] | ((uint32_t)p[1] << 8) | ((uint32_t)p[2] << 16) | ((uint32_t)p[3] << 24);
}

static uint16_t fen_zip_u16(const unsigned char* p) {
    return (uint16_t)((uint16_t)p[0] | ((uint16_t)p[1] << 8));
}

static void fen_zip_put16(unsigned char* p, uint16_t v) {
    p[0] = (unsigned char)(v & 0xFF);
    p[1] = (unsigned char)(v >> 8);
}

static void fen_zip_put32(unsigned char* p, uint32_t v) {
    p[0] = (unsigned char)(v & 0xFF);
    p[1] = (unsigned char)((v >> 8) & 0xFF);
    p[2] = (unsigned char)((v >> 16) & 0xFF);
    p[3] = (unsigned char)((v >> 24) & 0xFF);
}

/* Whole file into a fresh buffer; NULL on failure. */
static unsigned char* fen_zip_read_all(const char* path, size_t* out_len) {
    FILE* f = fopen(path, "rb");
    if (!f) return NULL;
    if (fseek(f, 0, SEEK_END) != 0) {
        fclose(f);
        return NULL;
    }
    long n = ftell(f);
    if (n < 0) {
        fclose(f);
        return NULL;
    }
    rewind(f);
    unsigned char* buf = (unsigned char*)rt_alloc((size_t)n + 1);
    size_t got = fread(buf, 1, (size_t)n, f);
    fclose(f);
    if (got != (size_t)n) return NULL;
    buf[got] = 0;
    *out_len = got;
    return buf;
}

static bool fen_zip_mkdirs(const char* path) {
    char* tmp = (char*)rt_dup(path);
    size_t n = strlen(tmp);
    for (size_t i = 1; i < n; i++) {
        if (!fen_zip_sep(tmp[i])) continue;
        char saved = tmp[i];
        tmp[i] = 0;
        if (tmp[0]) fen_zip_mkdir1(tmp);
        tmp[i] = saved;
    }
    bool ok = fen_zip_mkdir1(tmp) == 0;
    struct stat st;
    if (!ok && stat(tmp, &st) == 0 && S_ISDIR(st.st_mode)) ok = true;
    return ok;
}

/* Creates every parent directory of `path` (the file itself is not made). */
static bool fen_zip_mkparents(const char* path) {
    size_t n = strlen(path);
    size_t cut = n;
    while (cut > 0 && !fen_zip_sep(path[cut - 1])) cut--;
    if (cut == 0) return true;
    char* dir = (char*)rt_alloc(cut + 1);
    memcpy(dir, path, cut);
    dir[cut] = 0;
    /* drop the trailing separator so mkdir matches the directory itself */
    while (cut > 1 && fen_zip_sep(dir[cut - 1])) dir[--cut] = 0;
    if (cut == 0 || (cut == 1 && fen_zip_sep(dir[0]))) return true;
#ifdef _WIN32
    if (cut == 2 && dir[1] == ':') return true;
#endif
    return fen_zip_mkdirs(dir);
}

/* ---- inflate (RFC 1951) ------------------------------------------------ */

typedef struct {
    const unsigned char* in;
    size_t len;
    size_t pos;   /* byte position */
    uint32_t bitbuf;
    int bitcnt;
    unsigned char* out;
    size_t out_len;
    size_t out_cap;
    bool failed;
} FenInflate;

static void fen_inf_put(FenInflate* s, unsigned char b) {
    if (s->out_len == s->out_cap) {
        size_t nc = s->out_cap ? s->out_cap * 2 : 1024;
        s->out = (unsigned char*)rt_realloc(s->out, nc);
        s->out_cap = nc;
    }
    s->out[s->out_len++] = b;
}

static int fen_inf_bits(FenInflate* s, int need) {
    while (s->bitcnt < need) {
        if (s->pos >= s->len) {
            s->failed = true;
            return 0;
        }
        s->bitbuf |= (uint32_t)s->in[s->pos++] << s->bitcnt;
        s->bitcnt += 8;
    }
    int v = (int)(s->bitbuf & ((1u << need) - 1));
    s->bitbuf >>= need;
    s->bitcnt -= need;
    return v;
}

typedef struct {
    uint16_t count[16];
    uint16_t symbol[288];
} FenHuff;

/* Builds a canonical Huffman table; returns 0 for an over-subscribed code. */
static int fen_inf_build(FenHuff* h, const short* lengths, int n) {
    memset(h->count, 0, sizeof h->count);
    for (int i = 0; i < n; i++) h->count[lengths[i]]++;
    if (h->count[0] == n) return 0;
    int left = 1;
    for (int len = 1; len < 16; len++) {
        left <<= 1;
        left -= h->count[len];
        if (left < 0) return -1;
    }
    uint16_t offs[16];
    offs[1] = 0;
    for (int len = 1; len < 15; len++) offs[len + 1] = (uint16_t)(offs[len] + h->count[len]);
    for (int i = 0; i < n; i++) {
        if (lengths[i]) h->symbol[offs[lengths[i]]++] = (uint16_t)i;
    }
    return left;
}

static int fen_inf_decode(FenInflate* s, const FenHuff* h) {
    int code = 0, first = 0, index = 0;
    for (int len = 1; len < 16; len++) {
        code |= fen_inf_bits(s, 1);
        if (s->failed) return -1;
        int count = h->count[len];
        if (code - count < first) return (int)h->symbol[index + (code - first)];
        index += count;
        first += count;
        first <<= 1;
        code <<= 1;
    }
    s->failed = true;
    return -1;
}

static const short fen_inf_len_base[29] = {3,  4,  5,  6,  7,  8,  9,  10, 11,  13,
                                           15, 17, 19, 23, 27, 31, 35, 43, 51,  59,
                                           67, 83, 99, 115, 131, 163, 195, 227, 258};
static const short fen_inf_len_extra[29] = {0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2,
                                            2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0};
static const short fen_inf_dist_base[30] = {1,    2,    3,    4,    5,    7,     9,     13,
                                            17,   25,   33,   49,   65,   97,    129,   193,
                                            257,  385,  513,  769,  1025, 1537,  2049,  3073,
                                            4097, 6145, 8193, 12289, 16385, 24577};
static const short fen_inf_dist_extra[30] = {0, 0, 0, 0, 1, 1, 2, 2,  3,  3,  4,  4,  5,  5,  6,
                                             6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13};

static void fen_inf_codes(FenInflate* s, const FenHuff* lencode, const FenHuff* distcode) {
    for (;;) {
        int sym = fen_inf_decode(s, lencode);
        if (sym < 0) return;
        if (sym < 256) {
            fen_inf_put(s, (unsigned char)sym);
        } else if (sym == 256) {
            return;
        } else {
            sym -= 257;
            if (sym >= 29) {
                s->failed = true;
                return;
            }
            int len = fen_inf_len_base[sym] + fen_inf_bits(s, fen_inf_len_extra[sym]);
            int dsym = fen_inf_decode(s, distcode);
            if (dsym < 0 || dsym >= 30) {
                s->failed = true;
                return;
            }
            size_t dist = (size_t)(fen_inf_dist_base[dsym] + fen_inf_bits(s, fen_inf_dist_extra[dsym]));
            if (dist > s->out_len) {
                s->failed = true;
                return;
            }
            for (int i = 0; i < len; i++) fen_inf_put(s, s->out[s->out_len - dist]);
        }
        if (s->failed) return;
    }
}

static void fen_inf_fixed(FenInflate* s) {
    static FenHuff lencode, distcode;
    static int ready = 0;
    if (!ready) {
        short lengths[288];
        int i = 0;
        for (; i < 144; i++) lengths[i] = 8;
        for (; i < 256; i++) lengths[i] = 9;
        for (; i < 280; i++) lengths[i] = 7;
        for (; i < 288; i++) lengths[i] = 8;
        fen_inf_build(&lencode, lengths, 288);
        for (i = 0; i < 30; i++) lengths[i] = 5;
        fen_inf_build(&distcode, lengths, 30);
        ready = 1;
    }
    fen_inf_codes(s, &lencode, &distcode);
}

static void fen_inf_dynamic(FenInflate* s) {
    static const short order[19] = {16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15};
    int nlen = fen_inf_bits(s, 5) + 257;
    int ndist = fen_inf_bits(s, 5) + 1;
    int ncode = fen_inf_bits(s, 4) + 4;
    if (s->failed || nlen > 286 || ndist > 30) {
        s->failed = true;
        return;
    }
    short lengths[320];
    memset(lengths, 0, sizeof lengths);
    for (int i = 0; i < ncode; i++) lengths[order[i]] = (short)fen_inf_bits(s, 3);
    FenHuff lencode, distcode;
    if (fen_inf_build(&lencode, lengths, 19) != 0) {
        s->failed = true;
        return;
    }
    int index = 0;
    while (index < nlen + ndist) {
        int sym = fen_inf_decode(s, &lencode);
        if (sym < 0) {
            s->failed = true;
            return;
        }
        if (sym < 16) {
            lengths[index++] = (short)sym;
        } else {
            int len = 0, rep;
            if (sym == 16) {
                if (index == 0) {
                    s->failed = true;
                    return;
                }
                len = lengths[index - 1];
                rep = 3 + fen_inf_bits(s, 2);
            } else if (sym == 17) {
                rep = 3 + fen_inf_bits(s, 3);
            } else {
                rep = 11 + fen_inf_bits(s, 7);
            }
            if (index + rep > nlen + ndist) {
                s->failed = true;
                return;
            }
            while (rep--) lengths[index++] = (short)len;
        }
    }
    if (lengths[256] == 0) {
        s->failed = true;
        return;
    }
    if (fen_inf_build(&lencode, lengths, nlen) < 0 ||
        fen_inf_build(&distcode, lengths + nlen, ndist) < 0) {
        s->failed = true;
        return;
    }
    fen_inf_codes(s, &lencode, &distcode);
}

/* Decompresses a raw DEFLATE stream. Returns a buffer (NUL-terminated) and
 * its length in *out_len, or NULL on a corrupt stream. */
static unsigned char* fen_inflate(const unsigned char* in, size_t len, size_t* out_len) {
    FenInflate s;
    memset(&s, 0, sizeof s);
    s.in = in;
    s.len = len;
    int last;
    do {
        last = fen_inf_bits(&s, 1);
        int type = fen_inf_bits(&s, 2);
        if (s.failed) break;
        if (type == 0) {
            /* stored: drop the remaining bits of the current byte */
            s.bitbuf = 0;
            s.bitcnt = 0;
            if (s.pos + 4 > s.len) {
                s.failed = true;
                break;
            }
            unsigned int n = (unsigned int)fen_zip_u16(s.in + s.pos);
            unsigned int nn = (unsigned int)fen_zip_u16(s.in + s.pos + 2);
            s.pos += 4;
            if ((n ^ 0xFFFFu) != nn || s.pos + n > s.len) {
                s.failed = true;
                break;
            }
            for (unsigned int i = 0; i < n; i++) fen_inf_put(&s, s.in[s.pos + i]);
            s.pos += n;
        } else if (type == 1) {
            fen_inf_fixed(&s);
        } else if (type == 2) {
            fen_inf_dynamic(&s);
        } else {
            s.failed = true;
            break;
        }
    } while (!last && !s.failed);

    if (s.failed) return NULL;
    if (!s.out) {
        s.out = (unsigned char*)rt_alloc(1);
        s.out_cap = 1;
    }
    if (s.out_len == s.out_cap) s.out = (unsigned char*)rt_realloc(s.out, s.out_cap + 1);
    s.out[s.out_len] = 0;
    *out_len = s.out_len;
    return s.out;
}

/* ---- reading ---------------------------------------------------------- */

typedef struct {
    char* name;
    uint32_t compressed;
    uint32_t uncompressed;
    uint32_t crc;
    uint32_t offset;
    uint16_t method;
} FenZipEntry;

typedef struct {
    unsigned char* data;
    size_t len;
    FenZipEntry* entries;
    int count;
} FenZip;

static void fen_zip_free(FenZip* z) {
    for (int i = 0; i < z->count; i++) free(z->entries[i].name);
    free(z->entries);
    free(z->data);
    z->entries = NULL;
    z->data = NULL;
    z->count = 0;
}

/* Finds the end-of-central-directory record and loads the entry table. */
static bool fen_zip_open(FenZip* z, const char* path) {
    memset(z, 0, sizeof *z);
    z->data = fen_zip_read_all(path, &z->len);
    if (!z->data) {
        fen_zip_last_error = "cannot open the archive";
        return false;
    }
    if (z->len < 22) {
        fen_zip_last_error = "not a zip archive (too small)";
        return false;
    }
    size_t eocd = (size_t)-1;
    size_t lowest = z->len > 66 ? z->len - 66 : 0;
    for (size_t i = z->len - 22 + 1; i-- > lowest;) {
        if (fen_zip_u32(z->data + i) == 0x06054b50u) {
            eocd = i;
            break;
        }
    }
    if (eocd == (size_t)-1) {
        fen_zip_last_error = "not a zip archive (no end-of-central-directory)";
        return false;
    }
    unsigned int total = fen_zip_u16(z->data + eocd + 10);
    uint32_t cd_size = fen_zip_u32(z->data + eocd + 12);
    uint32_t cd_off = fen_zip_u32(z->data + eocd + 16);
    if (total == 0xFFFFu || cd_off == 0xFFFFFFFFu) {
        fen_zip_last_error = "zip64 archives are not supported";
        return false;
    }
    if ((size_t)cd_off + cd_size > z->len) {
        fen_zip_last_error = "truncated zip archive";
        return false;
    }
    z->entries = (FenZipEntry*)rt_alloc((total ? total : 1) * sizeof(FenZipEntry));
    size_t p = cd_off;
    for (unsigned int i = 0; i < total; i++) {
        if (p + 46 > z->len || fen_zip_u32(z->data + p) != 0x02014b50u) {
            fen_zip_last_error = "corrupt central directory";
            return false;
        }
        FenZipEntry e;
        e.method = fen_zip_u16(z->data + p + 10);
        e.crc = fen_zip_u32(z->data + p + 16);
        e.compressed = fen_zip_u32(z->data + p + 20);
        e.uncompressed = fen_zip_u32(z->data + p + 24);
        size_t nlen = fen_zip_u16(z->data + p + 28);
        size_t elen = fen_zip_u16(z->data + p + 30);
        size_t clen = fen_zip_u16(z->data + p + 32);
        e.offset = fen_zip_u32(z->data + p + 42);
        if (p + 46 + nlen > z->len) {
            fen_zip_last_error = "corrupt central directory";
            return false;
        }
        e.name = (char*)rt_alloc(nlen + 1);
        memcpy(e.name, z->data + p + 46, nlen);
        e.name[nlen] = 0;
        z->entries[z->count++] = e;
        p += 46 + nlen + elen + clen;
    }
    return true;
}

/* Decompressed bytes of `e` (NUL-terminated), NULL on failure. */
static unsigned char* fen_zip_bytes(FenZip* z, const FenZipEntry* e, size_t* out_len) {
    if ((size_t)e->offset + 30 > z->len || fen_zip_u32(z->data + e->offset) != 0x04034b50u) {
        fen_zip_last_error = "corrupt local header";
        return NULL;
    }
    size_t nlen = fen_zip_u16(z->data + e->offset + 26);
    size_t elen = fen_zip_u16(z->data + e->offset + 28);
    size_t start = (size_t)e->offset + 30 + nlen + elen;
    if (start + e->compressed > z->len) {
        fen_zip_last_error = "truncated entry";
        return NULL;
    }
    if (e->method == 0) {
        if (e->compressed != e->uncompressed) {
            fen_zip_last_error = "corrupt stored entry";
            return NULL;
        }
        unsigned char* buf = (unsigned char*)rt_alloc((size_t)e->compressed + 1);
        memcpy(buf, z->data + start, e->compressed);
        buf[e->compressed] = 0;
        *out_len = e->compressed;
        return buf;
    }
    if (e->method == 8) {
        unsigned char* buf = fen_inflate(z->data + start, e->compressed, out_len);
        if (!buf) {
            fen_zip_last_error = "corrupt deflate stream";
            return NULL;
        }
        return buf;
    }
    fen_zip_last_error = "unsupported compression method (only stored and deflate)";
    return NULL;
}

static FenZipEntry* fen_zip_find(FenZip* z, const char* name) {
    for (int i = 0; i < z->count; i++) {
        if (strcmp(z->entries[i].name, name) == 0) return &z->entries[i];
    }
    return NULL;
}

/* An entry name that is safe to join onto a destination directory: no
 * leading separator, no drive letter, and no ".." component, so extracting
 * can never write outside the folder it was given. Empty components and "."
 * are dropped. */
static char* fen_zip_safe_name(const char* name) {
    size_t n = strlen(name);
    char* out = (char*)rt_alloc(n + 1);
    size_t w = 0;
    bool trailing_sep = false;
    size_t i = 0;
#ifdef _WIN32
    if (n >= 2 && name[1] == ':') i = 2; /* skip "C:" */
#endif
    while (i < n) {
        while (i < n && fen_zip_sep(name[i])) i++;
        size_t start = i;
        while (i < n && !fen_zip_sep(name[i])) i++;
        size_t len = i - start;
        if (len == 0) continue;
        if (len == 1 && name[start] == '.') continue;
        if (len == 2 && name[start] == '.' && name[start + 1] == '.') {
            /* drop the ".." and the component it would escape into */
            while (w > 0 && out[w - 1] != '/') w--;
            if (w > 0) w--;
            trailing_sep = i < n;
            continue;
        }
        if (w > 0) out[w++] = '/';
        memcpy(out + w, name + start, len);
        w += len;
        trailing_sep = i < n;
    }
    if (trailing_sep && w > 0) out[w++] = '/';
    out[w] = 0;
    return out;
}

/* ---- public functions -------------------------------------------------- */

/* Entry names, one per line ("" when the archive cannot be read). */
FEN_UNUSED static const char* fen_zip_list(const char* archive) {
    FenZip z;
    if (!fen_zip_open(&z, archive)) return "";
    size_t cap = 256, len = 0;
    char* out = (char*)rt_alloc(cap);
    out[0] = 0;
    for (int i = 0; i < z.count; i++) {
        size_t n = strlen(z.entries[i].name);
        while (len + n + 2 > cap) cap *= 2;
        out = (char*)rt_realloc(out, cap);
        if (len) out[len++] = '\n';
        memcpy(out + len, z.entries[i].name, n);
        len += n;
        out[len] = 0;
    }
    fen_zip_free(&z);
    return out;
}

FEN_UNUSED static bool fen_zip_contains(const char* archive, const char* name) {
    FenZip z;
    if (!fen_zip_open(&z, archive)) return false;
    bool found = fen_zip_find(&z, name) != NULL;
    fen_zip_free(&z);
    return found;
}

/* Uncompressed size of an entry, or -1 when it is not there. */
FEN_UNUSED static int32_t fen_zip_entry_size(const char* archive, const char* name) {
    FenZip z;
    if (!fen_zip_open(&z, archive)) return -1;
    FenZipEntry* e = fen_zip_find(&z, name);
    int32_t size = e ? (int32_t)e->uncompressed : -1;
    fen_zip_free(&z);
    return size;
}

/* Contents of one entry ("" when it is missing or cannot be read). */
FEN_UNUSED static const char* fen_zip_read(const char* archive, const char* name) {
    FenZip z;
    if (!fen_zip_open(&z, archive)) return "";
    FenZipEntry* e = fen_zip_find(&z, name);
    if (!e) {
        fen_zip_last_error = "no such entry";
        fen_zip_free(&z);
        return "";
    }
    size_t len = 0;
    unsigned char* buf = fen_zip_bytes(&z, e, &len);
    fen_zip_free(&z);
    if (!buf) return "";
    return (const char*)buf;
}

/* Writes one entry to `dest`, creating parent directories. */
FEN_UNUSED static bool fen_zip_extract_file(const char* archive, const char* name,
                                            const char* dest) {
    FenZip z;
    if (!fen_zip_open(&z, archive)) return false;
    FenZipEntry* e = fen_zip_find(&z, name);
    if (!e) {
        fen_zip_last_error = "no such entry";
        fen_zip_free(&z);
        return false;
    }
    char* safe = fen_zip_safe_name(e->name);
    if (strlen(safe) == 0) {
        fen_zip_last_error = "the entry name is not a usable path";
        free(safe);
        fen_zip_free(&z);
        return false;
    }
    free(safe);
    size_t len = 0;
    unsigned char* buf = fen_zip_bytes(&z, e, &len);
    if (!buf) {
        fen_zip_free(&z);
        return false;
    }
    bool ok = fen_zip_mkparents(dest);
    FILE* f = ok ? fopen(dest, "wb") : NULL;
    if (!f) {
        fen_zip_last_error = "cannot write the destination file";
        fen_zip_free(&z);
        return false;
    }
    ok = len == 0 || fwrite(buf, 1, len, f) == len;
    if (fclose(f) != 0) ok = false;
    if (!ok) fen_zip_last_error = "cannot write the destination file";
    fen_zip_free(&z);
    return ok;
}

/* Extracts every entry under `dest_dir`. */
FEN_UNUSED static bool fen_zip_extract(const char* archive, const char* dest_dir) {
    FenZip z;
    if (!fen_zip_open(&z, archive)) return false;
    if (!fen_zip_mkdirs(dest_dir)) {
        fen_zip_last_error = "cannot create the destination directory";
        fen_zip_free(&z);
        return false;
    }
    bool ok = true;
    size_t dl = strlen(dest_dir);
    bool sep = dl > 0 && fen_zip_sep(dest_dir[dl - 1]);
    for (int i = 0; i < z.count && ok; i++) {
        FenZipEntry* e = &z.entries[i];
        /* Entry names come from the archive, so they are sanitised first:
           "../x" and "/abs" must stay inside dest_dir. */
        char* rel = fen_zip_safe_name(e->name);
        size_t n = strlen(rel);
        if (n == 0) {
            free(rel);
            continue;
        }
        char* path = (char*)rt_alloc(dl + n + 2);
        snprintf(path, dl + n + 2, "%s%s%s", dest_dir, sep ? "" : "/", rel);
        /* a directory entry just makes the folder */
        if (fen_zip_sep(rel[n - 1])) {
            ok = fen_zip_mkdirs(path);
            free(rel);
            continue;
        }
        size_t len = 0;
        unsigned char* buf = fen_zip_bytes(&z, e, &len);
        if (!buf) {
            ok = false;
            free(rel);
            break;
        }
        ok = fen_zip_mkparents(path);
        FILE* f = ok ? fopen(path, "wb") : NULL;
        if (!f) {
            fen_zip_last_error = "cannot write an extracted file";
            ok = false;
            free(rel);
            break;
        }
        ok = len == 0 || fwrite(buf, 1, len, f) == len;
        if (fclose(f) != 0) ok = false;
        if (!ok) fen_zip_last_error = "cannot write an extracted file";
        free(rel);
    }
    fen_zip_free(&z);
    return ok;
}

/* ---- writing ---------------------------------------------------------- */

typedef struct {
    unsigned char* data;
    size_t len;
    size_t cap;
} FenBuf;

static void fen_buf_add(FenBuf* b, const void* p, size_t n) {
    while (b->len + n > b->cap) {
        size_t nc = b->cap ? b->cap * 2 : 1024;
        while (nc < b->len + n) nc *= 2;
        b->data = (unsigned char*)rt_realloc(b->data, nc);
        b->cap = nc;
    }
    memcpy(b->data + b->len, p, n);
    b->len += n;
}

static void fen_buf_u16(FenBuf* b, uint16_t v) {
    unsigned char t[2];
    fen_zip_put16(t, v);
    fen_buf_add(b, t, 2);
}

static void fen_buf_u32(FenBuf* b, uint32_t v) {
    unsigned char t[4];
    fen_zip_put32(t, v);
    fen_buf_add(b, t, 4);
}

/* The DOS date/time pair that zip entries carry. */
static void fen_zip_dos_time(uint16_t* out_date, uint16_t* out_time) {
    time_t now = time(NULL);
    struct tm* t = localtime(&now);
    if (!t) {
        *out_date = (uint16_t)(1 << 5 | 1); /* 1980-01-01 */
        *out_time = 0;
        return;
    }
    int year = t->tm_year + 1900;
    if (year < 1980) year = 1980;
    *out_date = (uint16_t)(((year - 1980) & 0x7F) << 9 | ((t->tm_mon + 1) & 0x0F) << 5 |
                           (t->tm_mday & 0x1F));
    *out_time = (uint16_t)((t->tm_hour & 0x1F) << 11 | (t->tm_min & 0x3F) << 5 |
                           ((t->tm_sec / 2) & 0x1F));
}

/* Writes an archive with `count` entries; names[i] is stored, sources[i] read
 * from disk. An empty source stores an empty entry. */
FEN_UNUSED static bool fen_zip_create(const char* archive, const char* names,
                                      const char* sources) {
    fen_zip_last_error = "";
    FenBuf out = {0, 0, 0};
    FenBuf central = {0, 0, 0};
    uint16_t ddate, dtime;
    fen_zip_dos_time(&ddate, &dtime);

    const char* np = names ? names : "";
    const char* sp = sources ? sources : "";
    int count = 0;
    bool ok = true;

    while (*np || *sp) {
        const char* ne = strchr(np, '\n');
        size_t nlen = ne ? (size_t)(ne - np) : strlen(np);
        const char* se = strchr(sp, '\n');
        size_t slen = se ? (size_t)(se - sp) : strlen(sp);

        char name[1024];
        if (nlen >= sizeof name) {
            fen_zip_last_error = "entry name too long";
            ok = false;
            break;
        }
        memcpy(name, np, nlen);
        name[nlen] = 0;

        char src[1024];
        if (slen >= sizeof src) {
            fen_zip_last_error = "source path too long";
            ok = false;
            break;
        }
        memcpy(src, sp, slen);
        src[slen] = 0;

        size_t data_len = 0;
        unsigned char* data = NULL;
        if (slen > 0) {
            data = fen_zip_read_all(src, &data_len);
            if (!data) {
                fen_zip_last_error = "cannot read a file to add to the archive";
                ok = false;
                break;
            }
        } else {
            data = (unsigned char*)rt_alloc(1);
            data[0] = 0;
        }
        if (data_len > 0xFFFFFFFFu || out.len > 0xFFFFFFFFu) {
            fen_zip_last_error = "archive too large";
            ok = false;
            break;
        }
        uint32_t crc = fen_zip_crc32(data, data_len);
        uint32_t offset = (uint32_t)out.len;

        /* local file header, then the raw bytes (stored) */
        fen_buf_u32(&out, 0x04034b50u);
        fen_buf_u16(&out, 20);
        fen_buf_u16(&out, 0);
        fen_buf_u16(&out, 0); /* method 0: stored */
        fen_buf_u16(&out, dtime);
        fen_buf_u16(&out, ddate);
        fen_buf_u32(&out, crc);
        fen_buf_u32(&out, (uint32_t)data_len);
        fen_buf_u32(&out, (uint32_t)data_len);
        fen_buf_u16(&out, (uint16_t)nlen);
        fen_buf_u16(&out, 0);
        fen_buf_add(&out, name, nlen);
        fen_buf_add(&out, data, data_len);

        /* matching central directory record */
        fen_buf_u32(&central, 0x02014b50u);
        fen_buf_u16(&central, 20);
        fen_buf_u16(&central, 20);
        fen_buf_u16(&central, 0);
        fen_buf_u16(&central, 0);
        fen_buf_u16(&central, dtime);
        fen_buf_u16(&central, ddate);
        fen_buf_u32(&central, crc);
        fen_buf_u32(&central, (uint32_t)data_len);
        fen_buf_u32(&central, (uint32_t)data_len);
        fen_buf_u16(&central, (uint16_t)nlen);
        fen_buf_u16(&central, 0);
        fen_buf_u16(&central, 0);
        fen_buf_u16(&central, 0);
        fen_buf_u16(&central, 0);
        fen_buf_u32(&central, 0);
        fen_buf_u32(&central, offset);
        fen_buf_add(&central, name, nlen);

        free(data);
        count++;
        np += nlen;
        if (*np == '\n') np++;
        sp += slen;
        if (*sp == '\n') sp++;
        if (np == names + strlen(names) && sp == sources + strlen(sources)) break;
    }

    if (ok) {
        if (count > 0xFFFF) {
            fen_zip_last_error = "too many entries";
            ok = false;
        } else {
            uint32_t cd_off = (uint32_t)out.len;
            fen_buf_add(&out, central.data, central.len);
            fen_buf_u32(&out, 0x06054b50u);
            fen_buf_u16(&out, 0);
            fen_buf_u16(&out, 0);
            fen_buf_u16(&out, (uint16_t)count);
            fen_buf_u16(&out, (uint16_t)count);
            fen_buf_u32(&out, (uint32_t)central.len);
            fen_buf_u32(&out, cd_off);
            fen_buf_u16(&out, 0);

            FILE* f = fopen(archive, "wb");
            if (!f) {
                fen_zip_last_error = "cannot write the archive";
                ok = false;
            } else {
                ok = out.len == 0 || fwrite(out.data, 1, out.len, f) == out.len;
                if (fclose(f) != 0) ok = false;
                if (!ok) fen_zip_last_error = "cannot write the archive";
            }
        }
    }
    free(out.data);
    free(central.data);
    return ok;
}
