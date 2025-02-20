use criterion::{Criterion, black_box, criterion_group, criterion_main};

const BASE64_CHARS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789+/";

fn decode_base64_find(input: &str) -> String {
    let mut decoded = String::new();
    let mut buffer: u32 = 0;
    let mut bits: u32 = 0;

    for char in input.chars() {
        if let Some(value) = BASE64_CHARS.find(char) {
            buffer = (buffer << 6) | (value as u32);
            bits += 6;

            while bits >= 8 {
                bits -= 8;
                let byte = ((buffer >> bits) & 0xFF) as u8;
                decoded.push(byte as char);
            }
        }
    }

    decoded
}

fn decode_base64_match(input: &str) -> String {
    let mut decoded = String::new();
    let mut buffer: u32 = 0;
    let mut bits: u32 = 0;

    for &byte in input.as_bytes() {
        let value = match byte {
            b'a'..=b'z' => byte - b'a',      // 0-25
            b'A'..=b'Z' => byte - b'A' + 26, // 26-51
            b'0'..=b'9' => byte - b'0' + 52, // 52-61
            b'+' => 62,
            b'/' => 63,
            _ => continue,
        };

        buffer = (buffer << 6) | (value as u32);
        bits += 6;

        while bits >= 8 {
            bits -= 8;
            let byte = ((buffer >> bits) & 0xFF) as u8;
            decoded.push(byte as char);
        }
    }

    decoded
}

fn criterion_benchmark(c: &mut Criterion) {
    let small_input = "SGVsbG8gV29ybGQ=";
    let medium_input = "TG9yZW0gaXBzdW0gZG9sb3Igc2l0IGFtZXQ=";
    let large_input = "TG9yZW0gaXBzdW0gZG9sb3Igc2l0IGFtZXQsIGNvbnNlY3RldHVyIGFkaXBpc2NpbmcgZWxpdC4gUXVpc3F1ZSB1bHRyaWNlcyBtYXVyaXMgZWdldCBtYXNzYSBjb252YWxsaXMsIHNlZCBwZWxsZW50ZXNxdWUgbGlndWxhIHNjZWxlcmlzcXVlLg==";

    let mut group = c.benchmark_group("Base64 Decode");

    group.bench_function("find_small", |b| b.iter(|| decode_base64_find(black_box(small_input))));
    group.bench_function("match_small", |b| b.iter(|| decode_base64_match(black_box(small_input))));

    group.bench_function("find_medium", |b| b.iter(|| decode_base64_find(black_box(medium_input))));
    group.bench_function("match_medium", |b| {
        b.iter(|| decode_base64_match(black_box(medium_input)))
    });

    group.bench_function("find_large", |b| b.iter(|| decode_base64_find(black_box(large_input))));
    group.bench_function("match_large", |b| b.iter(|| decode_base64_match(black_box(large_input))));

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
