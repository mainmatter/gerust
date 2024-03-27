use anyhow::anyhow;
use insta::assert_snapshot;
use pacesetter::ui::UI;

#[test]
fn test_no_color() {
    let mut stdout = create_buffer();
    let mut stderr = create_buffer();
    let mut ui = UI::new(&mut stdout, &mut stderr, false, false);
    ui.log("a general message");
    ui.info("an info message");
    ui.success("a success message ✓");
    ui.error("an error message :(", anyhow!("oh no…"));

    let output = read_buffer(stdout);
    let error_output = read_buffer(stderr);

    assert_snapshot!(output);
    assert_snapshot!(error_output);
}

#[test]
fn test_color() {
    let mut stdout = create_buffer();
    let mut stderr = create_buffer();
    let mut ui = UI::new(&mut stdout, &mut stderr, true, false);
    ui.log("a general message");
    ui.info("an info message");
    ui.success("a success message ✓");
    ui.error("an error message :(", anyhow!("oh no…"));

    let output = read_buffer(stdout);
    let error_output = read_buffer(stderr);

    assert_snapshot!(output);
    assert_snapshot!(error_output);
}

#[test]
fn test_indentation() {
    let mut stdout = create_buffer();
    let mut stderr = create_buffer();
    let mut ui = UI::new(&mut stdout, &mut stderr, true, true);
    ui.log("a log message");
    ui.indent();
    ui.log("an indented message");
    ui.indent();
    ui.indent();
    ui.indent();
    ui.log("more indentation");
    ui.outdent();
    ui.outdent();
    ui.log("less indentation");
    ui.outdent();
    ui.outdent();
    ui.outdent(); // we can call outdent more often than indent – it just stops at 0 indentation
    ui.log("no indentation");

    let output = read_buffer(stdout);

    assert_snapshot!(output);
}

fn create_buffer() -> std::io::BufWriter<Vec<u8>> {
    std::io::BufWriter::new(Vec::new())
}

fn read_buffer(buffer: std::io::BufWriter<Vec<u8>>) -> String {
    let bytes = buffer.into_inner().unwrap();

    String::from_utf8(bytes).unwrap()
}
