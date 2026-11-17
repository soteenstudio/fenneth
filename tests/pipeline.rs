use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn compile_ok(src: &str) -> String {
    fenneth::compile(src).expect("program should compile")
}

fn compile_err(src: &str) -> Vec<String> {
    fenneth::compile(src)
        .expect_err("program should be rejected")
        .into_iter()
        .map(|d| d.message)
        .collect()
}

#[test]
fn emits_c_for_hello() {
    let c = compile_ok("fn main() -> i32 { print(\"Hello Fenneth\"); return 0; }");
    assert!(c.contains("#include <stdio.h>"));
    assert!(c.contains("printf(\"%s\\n\", \"Hello Fenneth\")"));
    assert!(c.contains("int main(int argc, char** argv)"));
}

#[test]
fn string_constants_have_no_duplicate_const() {
    let c = compile_ok("fn main() -> i32 { const s: String = \"x\"; return 0; }");
    assert!(!c.contains("const const"));
    assert!(c.contains("const char* const v_s"));
}

#[test]
fn string_equality_uses_strcmp() {
    let c = compile_ok("fn main() -> i32 { let a: String = \"a\"; if a == \"b\" { } return 0; }");
    assert!(c.contains("strcmp(v_a, \"b\") == 0"));
}

#[test]
fn reports_multiple_type_errors() {
    let errs = compile_err("fn main() -> i32 { let a: i32 = true; let b: bool = 1; return 0; }");
    assert_eq!(errs.len(), 2);
}

#[test]
fn rejects_shadowing_and_undefined_names() {
    assert!(
        !compile_err("fn main() -> i32 { let a: i32 = 1; let a: i32 = 2; return 0; }").is_empty()
    );
    assert!(!compile_err("fn main() -> i32 { return nope; }").is_empty());
    assert!(!compile_err("fn main() -> i32 { return missing(); }").is_empty());
}

#[test]
fn rejects_wrong_call_arity_and_types() {
    let src = "fn f(a: i32) -> i32 { return a; } fn main() -> i32 { return f(1, 2); }";
    assert!(!compile_err(src).is_empty());
    let src = "fn f(a: i32) -> i32 { return a; } fn main() -> i32 { return f(true); }";
    assert!(!compile_err(src).is_empty());
}

#[test]
fn escapes_strings_for_c() {
    let c = compile_ok("fn main() -> i32 { print(\"a\\\"b\\n\"); return 0; }");
    assert!(c.contains("\"a\\\"b\\n\""));
}

/// Compiles an example to C, builds it with the system C compiler (when
/// present) and checks the program's output.
#[test]
fn demo_program_runs_correctly() {
    let cc = ["gcc", "clang", "cc"].into_iter().find(|c| {
        Command::new(c)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    });
    let Some(cc) = cc else {
        eprintln!("no C compiler available; skipping");
        return;
    };

    let src = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/examples/demo.fenneth"
    ))
    .unwrap();
    let c = compile_ok(&src);

    let dir = std::env::temp_dir().join(format!("fenneth-test-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let c_path = dir.join("demo.c");
    let exe = dir.join("demo_exe");
    fs::write(&c_path, c).unwrap();

    let build = Command::new(cc)
        .args(["-std=c99", "-Wall", "-Werror", "-o"])
        .arg(&exe)
        .arg(&c_path)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "C compile failed: {}",
        String::from_utf8_lossy(&build.stderr)
    );

    let run = Command::new(&exe).output().unwrap();
    assert!(run.status.success());
    let out = String::from_utf8_lossy(&run.stdout).to_string();
    let _ = fs::remove_dir_all(&dir);

    let expected = "Fenneth v0.1.0\n42\n22.5\n120\ntrue\nnegative\nsum of 1..10 = 55\nall good\n";
    assert_eq!(out, expected);
}

// ---------------------------------------------------------------------------
// v0.2.0: CLI, modules, runtime checks, self-hosted lexer, Rust FFI
// ---------------------------------------------------------------------------

fn fenneth(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_fenneth"))
        .args(args)
        .output()
        .expect("failed to launch the fenneth binary")
}

fn root(rel: &str) -> String {
    format!("{}/{}", env!("CARGO_MANIFEST_DIR"), rel)
}

fn have(tool: &str) -> bool {
    Command::new(tool)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn have_cc() -> bool {
    have("gcc") || have("clang") || have("cc")
}

fn scratch(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("fenneth-it-{}-{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

fn out(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).to_string()
}

fn err(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).to_string()
}

fn write(dir: &Path, name: &str, body: &str) -> String {
    let p = dir.join(name);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&p, body).unwrap();
    p.to_string_lossy().to_string()
}

#[test]
fn cli_version_and_file_shorthand() {
    let v = fenneth(&["--version"]);
    assert_eq!(out(&v).trim(), format!("fenneth {}", fenneth::VERSION));
    if !have_cc() {
        return;
    }
    // `fenneth <file>` runs it, and the extension may be omitted.
    let o = fenneth(&[&root("examples/hello.fenneth")]);
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(out(&o), "Hello Fenneth\n");
    let o = fenneth(&[&root("examples/hello")]);
    assert_eq!(out(&o), "Hello Fenneth\n");
}

#[test]
fn both_extensions_work_for_entry_files_and_imports() {
    if !have_cc() {
        return;
    }
    let d = scratch("ext");
    write(
        &d,
        "util.fenneth",
        "fn twice(x: i32) -> i32 { return x * 2; }",
    );
    write(&d, "more.fen", "fn thrice(x: i32) -> i32 { return x * 3; }");
    let main = write(
        &d,
        "main.fen",
        "import \"util\"; import \"more.fen\";\n\
         fn main() -> i32 { print(twice(4) + thrice(5)); return 0; }",
    );
    let o = fenneth(&["run", &main]);
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(out(&o), "23\n");
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn import_cycles_and_duplicates_are_loaded_once() {
    let d = scratch("cycle");
    write(&d, "a.fen", "import \"b\"; fn fa() -> i32 { return 1; }");
    write(&d, "b.fen", "import \"a\"; fn fb() -> i32 { return 2; }");
    let main = write(
        &d,
        "main.fen",
        "import \"a\"; import \"b\";\nfn main() -> i32 { return fa() + fb() - 3; }",
    );
    assert!(fenneth::compile_path(Path::new(&main)).is_ok());
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn diagnostics_name_the_file_they_come_from() {
    let d = scratch("diag");
    write(&d, "bad.fen", "fn oops() -> i32 { return \"x\"; }");
    let main = write(
        &d,
        "main.fen",
        "import \"bad\";\nfn main() -> i32 { return oops(); }",
    );
    let o = fenneth(&["check", &main]);
    assert!(!o.status.success());
    let e = err(&o);
    assert!(e.contains("bad.fen:1:"), "{}", e);
    assert!(e.contains("returning String"), "{}", e);

    let missing = write(
        &d,
        "m2.fen",
        "import \"nope\";\nfn main() -> i32 { return 0; }",
    );
    let o = fenneth(&["check", &missing]);
    assert!(err(&o).contains("cannot find module"), "{}", err(&o));
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn runtime_checks_panic_with_exit_code_101() {
    if !have_cc() {
        return;
    }
    let d = scratch("panic");
    let oob = write(
        &d,
        "oob.fen",
        "fn main() -> i32 { let v: [i32] = [1]; print(\"before\"); print(v[3]); return 0; }",
    );
    let o = fenneth(&["run", &oob]);
    assert_eq!(o.status.code(), Some(101));
    assert_eq!(out(&o), "before\n");
    assert!(
        err(&o).contains("index 3 out of bounds (length 1)"),
        "{}",
        err(&o)
    );

    let div = write(
        &d,
        "div.fen",
        "fn main() -> i32 { let z: i32 = len(args()); print(10 / z); return 0; }",
    );
    let o = fenneth(&["run", &div]);
    assert_eq!(o.status.code(), Some(101));
    assert!(err(&o).contains("division by zero"), "{}", err(&o));

    let user = write(
        &d,
        "p.fen",
        "fn main() -> i32 { panic(\"boom\"); return 0; }",
    );
    let o = fenneth(&["run", &user]);
    assert_eq!(o.status.code(), Some(101));
    assert!(err(&o).contains("panic: boom"), "{}", err(&o));
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn exit_code_and_program_args_pass_through() {
    if !have_cc() {
        return;
    }
    let d = scratch("args");
    let p = write(
        &d,
        "a.fen",
        "fn main() -> i32 { let a: [String] = args(); print(a[1] + a[0]); return len(a); }",
    );
    let o = fenneth(&["run", &p, "--", "x", "y"]);
    assert_eq!(out(&o), "yx\n");
    assert_eq!(o.status.code(), Some(2));
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn generated_c_is_warning_free_on_every_available_compiler() {
    let src = fs::read_to_string(root("examples/selfhost/lexer.fen")).unwrap();
    let main = fs::read_to_string(root("examples/selfhost/main.fen")).unwrap();
    let d = scratch("warn");
    write(&d, "lexer.fen", &src);
    let entry = write(&d, "main.fen", &main);
    let c = fenneth::compile_path(Path::new(&entry)).expect("selfhost sources compile");
    let c_path = write(&d, "out.c", &c);
    let mut tried = 0;
    for cc in ["gcc", "clang"] {
        if !have(cc) {
            continue;
        }
        tried += 1;
        let o = Command::new(cc)
            .args([
                "-std=c99",
                "-Wall",
                "-Wextra",
                "-Wno-unused-parameter",
                "-Werror",
                "-fwrapv",
                "-o",
            ])
            .arg(d.join("out_bin"))
            .arg(&c_path)
            .output()
            .unwrap();
        assert!(
            o.status.success(),
            "{} rejected the generated C:\n{}",
            cc,
            err(&o)
        );
    }
    let _ = tried;
    let _ = fs::remove_dir_all(&d);
}

/// The Fenneth lexer (examples/selfhost) must agree with the Rust lexer on
/// the token count and on the line:col of every token, for real sources.
#[test]
fn selfhosted_lexer_matches_rust_lexer() {
    if !have_cc() {
        return;
    }
    let d = scratch("selfhost");
    let exe = d.join("fen_lexer").to_string_lossy().to_string();
    let b = fenneth(&["build", &root("examples/selfhost/main.fen"), "-o", &exe]);
    assert!(b.status.success(), "{}", err(&b));

    let inputs = [
        "examples/hello.fenneth",
        "examples/demo.fenneth",
        "examples/selfhost/lexer.fen",
        "examples/selfhost/main.fen",
        "examples/ffi/ffi.fen",
    ];
    for rel in inputs {
        let path = root(rel);
        let src = fs::read_to_string(&path).unwrap();
        assert!(
            src.is_ascii(),
            "{} must be ASCII for the column comparison",
            rel
        );

        let expected: Vec<String> = fenneth::lexer::Lexer::new(&src)
            .tokenize()
            .unwrap()
            .iter()
            .map(|t| format!("{}:{}", t.pos.line, t.pos.col))
            .collect();

        let o = Command::new(&exe).args([&path, "--dump"]).output().unwrap();
        assert!(o.status.success(), "{}: {}", rel, err(&o));
        let text = out(&o);
        let mut lines: Vec<&str> = text.lines().collect();
        let summary = lines.pop().unwrap();
        let got: Vec<String> = lines
            .iter()
            .map(|l| l.split(' ').next().unwrap().to_string())
            .collect();

        assert_eq!(summary, format!("tokens: {}", expected.len()), "{}", rel);
        assert_eq!(got, expected, "token positions differ for {}", rel);
    }
    let _ = fs::remove_dir_all(&d);
}

#[cfg(unix)]
#[test]
fn rust_ffi_example_links_and_runs() {
    if !have_cc() || !have("rustc") {
        eprintln!("rustc or a C compiler is missing; skipping FFI test");
        return;
    }
    let d = scratch("ffi");
    let lib = d.join("libhost.a").to_string_lossy().to_string();
    let r = Command::new("rustc")
        .args(["--edition", "2021", "--crate-type", "staticlib", "-O"])
        .arg(root("examples/ffi/host.rs"))
        .args(["-o", &lib])
        .output()
        .unwrap();
    assert!(r.status.success(), "{}", err(&r));

    let o = fenneth(&[
        "run",
        &root("examples/ffi/ffi.fen"),
        "--link",
        &lib,
        "--link",
        "-lpthread",
        "--link",
        "-ldl",
        "--link",
        "-lm",
    ]);
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(out(&o), "42\n1.5\nHello from Rust, Fenneth!\n");
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn language_features_end_to_end() {
    if !have_cc() {
        return;
    }
    let d = scratch("features");
    let p = write(
        &d,
        "f.fen",
        r#"
        struct Point { x: i32, y: i32, }
        struct Shape { name: String, pts: [Point], }
        const LIMIT: i32 = 3;
        let counter: i32 = 0;
        fn bump() -> i32 { counter += 1; return counter; }
        fn main() -> i32 {
            let s: Shape = Shape { name: "tri", pts: [] };
            for i in 0..LIMIT { push(s.pts, Point { x: i, y: i * 2 }); }
            s.pts[1].x += 10;
            let total: i32 = 0;
            for p in s.pts {
                if p.x == 11 { continue; }
                total += p.x * p.x + p.y * p.y;
            }
            print(s.name + ":" + to_str(total) + ":" + to_str(len(s.pts)));
            let grid: [[i32]] = [[1, 2], [3]];
            push(grid[1], 9);
            print(to_str(len(grid[1])) + to_str(grid[1][1]));
            print(substr("hello", 1, 3));
            print(str_at("A", 0));
            print(bump() + bump());
            let w: i32 = -2147483648;
            print(w);
            print(0 - 7 % 3);
            let k: i32 = 0;
            while true { k += 1; if k > 4 { break; } }
            print(k);
            return 0;
        }
        "#,
    );
    let o = fenneth(&["run", &p]);
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(out(&o), "tri:20:3\n29\nel\n65\n3\n-2147483648\n-1\n5\n");
    let _ = fs::remove_dir_all(&d);
}
