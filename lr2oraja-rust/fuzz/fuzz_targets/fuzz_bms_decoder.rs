#![no_main]

use libfuzzer_sys::fuzz_target;

use bms_model::bms_decoder::BMSDecoder;

fuzz_target!(|data: &[u8]| {
    // Feed arbitrary bytes to the BMS decoder.
    // The decoder internally converts from Shift_JIS, so raw bytes are the right input.
    let mut decoder = BMSDecoder::new();
    let _ = decoder.decode_bytes(data, false, None);

    // Also test PMS mode parsing
    let mut decoder_pms = BMSDecoder::new();
    let _ = decoder_pms.decode_bytes(data, true, None);
});
