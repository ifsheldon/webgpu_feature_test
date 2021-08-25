use clap::Clap;
use std::fs;
use std::fs::read_dir;
use std::process::Command;

#[derive(Clap)]
struct Options {
    example_name: String,
}


fn main() {
    let options: Options = Options::parse();
    let example_dirs = read_dir("./examples")
        .expect("Should find the ./example")
        .map(|result| result.map(|entry| entry.file_name().into_string().unwrap()))
        .collect::<Result<Vec<_>, std::io::Error>>()
        .expect("Should find entries");
    let example_name = &options.example_name;
    if example_dirs.contains(example_name) {
        println!("Begin building example {}", example_name);
        let mut cargo_build_exec = Command::new("cargo")
            .env("RUSTFLAGS", "--cfg=web_sys_unstable_apis")
            .args(["build",
                "--target", "wasm32-unknown-unknown",
                "--example", example_name.as_str()])
            .spawn()
            .unwrap();
        cargo_build_exec.wait().expect("exec failed");
        println!("Generating wasm build");
        let artifact_path = format!("./generated/{}", example_name);
        let mut wasm_bindgen_exec = Command::new("wasm-bindgen")
            .args(["--out-dir", artifact_path.as_str(),
                "--web", format!("target/wasm32-unknown-unknown/debug/examples/{}.wasm", example_name).as_str()
            ])
            .spawn()
            .unwrap();
        wasm_bindgen_exec.wait().expect("wasm bindgen failed");
        let html_string = format!("
<!DOCTYPE html>
<html>
  <body>
    <script type=\"module\">
      import init from \"./{}.js\";
      init();
    </script>
  </body>
</html>", example_name);
        fs::write(format!("{}/index.html", artifact_path), html_string).expect("Cannot write html file");
        println!("Finished building all artifacts of example {}", example_name);
    } else {
        panic!("Please enter a correct example name. Got: {}", options.example_name);
    }
}
