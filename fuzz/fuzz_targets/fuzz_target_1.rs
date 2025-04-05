#![no_main]

use libfuzzer_sys::fuzz_target;
// Assuming your parser function is accessible via your project structure.
use borf::parser::parse_program;

fuzz_target!(|data: &[u8]| {
    // Try to convert the byte slice to a string.
    // If it's not valid UTF-8, that's okay, the parser should ideally handle it gracefully
    // or return an error. We just proceed to test the parser with whatever string we get.
    if let Ok(input_str) = Std::str::from_utf8(data) {
        // Call the function under test. We don't need to check the Result;
        // libFuzzer detects crashes/panics.
        let _ = parse_program(input_str);
    }
    // If from_utf8 fails, we just ignore this input. Alternatively, we could
    // try parsing the raw bytes if the parser supported it, but ours expects a string.
});
