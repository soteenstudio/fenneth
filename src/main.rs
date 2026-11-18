//! Fenneth command-line driver.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{exit, Command};

use fenneth::loader::{resolve_path, EXTENSIONS};
use fenneth::stdlib;
use fenneth::VERSION;

const USAGE: &str = "\
Fenneth compiler v{VERSION}  (source files: .fen or .fenneth)

USAGE:
    fenneth <file>                               Same as `fenneth run <file>`
    fenneth run   <file> [options] [-- args...]  Build and run (args go to the program)
    fenneth build <file> [-o out] [options]      Compile to a native executable
    fenneth emit  <file> [-o out.c]              Compile to C (stdout by default)
    fenneth check <file>                         Type-check only
    fenneth std [module]                         List the standard library, or print a module
    fenneth --version | --help

OPTIONS (build/run):
    --cc <compiler>    C compiler to use (default: $FENNETH_CC, gcc, clang, cc)
    --link <arg>       Extra argument for the C compiler/linker, repeatable
                       (e.g. --link ./libhost.a --link -lm --link -lpthread)
    --keep-c           Keep the generated .c file next to the output

`<file>` may omit its extension; `.fen` is tried before `.fenneth`.
The standard library is imported with `import \"std/<module>\";`.
";

struct Options {
    file: String,
    output: Option<String>,
    cc: Option<String>,
    link: Vec<String>,
    keep_c: bool,
    program_args: Vec<String>,
}

fn die(msg: &str) -> ! {
    eprintln!("error: {}", msg);
    exit(1);
}

fn usage() -> String {
    USAGE.replace("{VERSION}", VERSION)
}

fn parse_options(args: &[String]) -> Options {
    let mut file = None;
    let mut output = None;
    let mut cc = None;
    let mut link = Vec::new();
    let mut keep_c = false;
    let mut program_args = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--" => {
                program_args = args[i + 1..].to_vec();
                break;
            }
            "-o" => {
                i += 1;
                output = Some(
                    args.get(i)
                        .cloned()
                        .unwrap_or_else(|| die("-o needs a value")),
                );
            }
            "--cc" => {
                i += 1;
                cc = Some(
                    args.get(i)
                        .cloned()
                        .unwrap_or_else(|| die("--cc needs a value")),
                );
            }
            "--link" => {
                i += 1;
                link.push(
                    args.get(i)
                        .cloned()
                        .unwrap_or_else(|| die("--link needs a value")),
                );
            }
            "--keep-c" => keep_c = true,
            other if other.starts_with('-') => die(&format!("unknown option '{}'", other)),
            other => {
                if file.is_some() {
                    die("only one input file is supported (use `import` for more)");
                }
                file = Some(other.to_string());
            }
        }
        i += 1;
    }
    Options {
        file: file.unwrap_or_else(|| die("no input file given (see --help)")),
        output,
        cc,
        link,
        keep_c,
        program_args,
    }
}

/// Compiles a source file (and its imports), printing diagnostics on failure.
fn compile_file(path: &str) -> String {
    match fenneth::compile_path(Path::new(path)) {
        Ok(c) => c,
        Err(err) => {
            for line in err.render() {
                eprintln!("{}", line);
            }
            let n = err.diagnostics.len();
            eprintln!("aborting: {} error{}", n, if n == 1 { "" } else { "s" });
            exit(1);
        }
    }
}

fn find_c_compiler(preferred: Option<&str>) -> String {
    if let Some(cc) = preferred {
        return cc.to_string();
    }
    if let Ok(cc) = env::var("FENNETH_CC") {
        if !cc.is_empty() {
            return cc;
        }
    }
    for cand in ["gcc", "clang", "cc"] {
        let ok = Command::new(cand)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if ok {
            return cand.to_string();
        }
    }
    die("no C compiler found; install gcc or clang (Termux: `pkg install clang`, Windows: MSYS2/LLVM), or pass --cc <compiler>");
}

fn default_exe_name(input: &str) -> PathBuf {
    let stem = Path::new(input)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "a".into());
    if cfg!(windows) {
        PathBuf::from(format!("{}.exe", stem))
    } else {
        PathBuf::from(stem)
    }
}

/// Compiles `opts.file` to a native executable at `exe`, writing the
/// intermediate C source to `c_path` (removed unless `--keep-c`).
fn build_native(opts: &Options, exe: &Path, c_path: &Path) {
    let c_source = compile_file(&opts.file);
    fs::write(c_path, c_source)
        .unwrap_or_else(|e| die(&format!("cannot write '{}': {}", c_path.display(), e)));
    let cc = find_c_compiler(opts.cc.as_deref());
    let status = Command::new(&cc)
        .args(["-std=c99", "-O2", "-fwrapv", "-w", "-o"])
        .arg(exe)
        .arg(c_path)
        .args(&opts.link)
        // The standard library (std/math) uses libm; harmless otherwise.
        .args(if cfg!(windows) { &[][..] } else { &["-lm"][..] })
        .status()
        .unwrap_or_else(|e| die(&format!("failed to run '{}': {}", cc, e)));
    if !opts.keep_c {
        let _ = fs::remove_file(c_path);
    }
    if !status.success() {
        die(&format!("C compiler '{}' failed", cc));
    }
}

fn cmd_run(rest: &[String]) -> ! {
    let opts = parse_options(rest);
    let dir = env::temp_dir().join(format!("fenneth-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap_or_else(|e| die(&format!("cannot create temp dir: {}", e)));
    let exe = dir.join(default_exe_name(&opts.file));
    let c_path = dir.join("program.c");
    build_native(&opts, &exe, &c_path);
    let status = Command::new(&exe)
        .args(&opts.program_args)
        .status()
        .unwrap_or_else(|e| die(&format!("failed to run program: {}", e)));
    let _ = fs::remove_dir_all(&dir);
    exit(status.code().unwrap_or(1));
}

/// `fenneth std` lists the standard modules; `fenneth std math` prints one.
fn cmd_std(rest: &[String]) {
    match rest.first() {
        None => {
            println!("Standard library modules (use `import \"std/<module>\";`):");
            for name in stdlib::module_names() {
                println!("    std/{}", name);
            }
            println!("\nPrint a module's source with `fenneth std <module>`.");
        }
        Some(arg) => {
            let name = arg.strip_prefix(stdlib::PREFIX).unwrap_or(arg);
            match stdlib::module_source(name) {
                Some(src) => print!("{}", src),
                None => die(&format!(
                    "unknown standard library module '{}' (available: {})",
                    arg,
                    stdlib::module_names().join(", ")
                )),
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let cmd = match args.first() {
        Some(c) => c.as_str(),
        None => {
            print!("{}", usage());
            exit(1);
        }
    };
    let rest = &args[1..];

    match cmd {
        "--version" | "-V" | "version" => println!("fenneth {}", VERSION),
        "--help" | "-h" | "help" => print!("{}", usage()),
        "emit" => {
            let opts = parse_options(rest);
            let c = compile_file(&opts.file);
            match opts.output {
                Some(out) => fs::write(&out, c)
                    .unwrap_or_else(|e| die(&format!("cannot write '{}': {}", out, e))),
                None => print!("{}", c),
            }
        }
        "check" => {
            let opts = parse_options(rest);
            compile_file(&opts.file);
            eprintln!("ok: {}", opts.file);
        }
        "build" => {
            let opts = parse_options(rest);
            let exe = opts
                .output
                .as_ref()
                .map(PathBuf::from)
                .unwrap_or_else(|| default_exe_name(&opts.file));
            let c_path = exe.with_extension("c");
            build_native(&opts, &exe, &c_path);
            eprintln!("built {}", exe.display());
        }
        "run" => cmd_run(rest),
        "std" => cmd_std(rest),
        other => {
            // `fenneth file.fen [args]` is shorthand for `fenneth run file.fen`.
            if !other.starts_with('-') && resolve_path(Path::new(other)).is_some() {
                cmd_run(&args);
            }
            eprintln!(
                "error: unknown command or file '{}' (source files end in .{})\n",
                other,
                EXTENSIONS.join(" or .")
            );
            eprint!("{}", usage());
            exit(1);
        }
    }
}
