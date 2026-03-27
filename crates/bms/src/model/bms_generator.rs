use crate::model::bms_decoder::BMSDecoder;
use crate::model::bms_model::BMSModel;

pub struct BMSGenerator {
    random: Vec<i32>,
    data: Vec<u8>,
    ispms: bool,
}

impl BMSGenerator {
    pub fn new(data: Vec<u8>, ispms: bool, random: Vec<i32>) -> Self {
        BMSGenerator {
            data,
            random,
            ispms,
        }
    }

    pub fn generate(&self, random: Option<&[i32]>) -> Option<BMSModel> {
        let mut decoder = BMSDecoder::new();
        decoder.decode_bytes(&self.data, self.ispms, random)
    }

    pub fn random(&self) -> &[i32] {
        &self.random
    }
}
