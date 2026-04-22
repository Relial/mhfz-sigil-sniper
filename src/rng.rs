#[derive(Clone, Copy)]
pub struct Lcg(u32);

impl Lcg {
    pub fn new(seed: u32) -> Self {
        Self(seed)
    }

    #[inline(always)]
    pub fn advance(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(0x343fd).wrapping_add(0x269ec3);
        self.0
    }

    #[inline(always)]
    pub fn advance_n(&mut self, n: usize) -> u32 {
        for _ in 0..n {
            self.advance();
        }
        self.0
    }

    #[inline(always)]
    pub fn low_byte(&self) -> u8 {
        (self.0 & 0xFF) as u8
    }

    pub fn peek_low_byte(&self, advances: usize) -> u8 {
        if advances == 0 {
            self.low_byte()
        } else {
            let mut temp = *self;
            for _ in 0..advances {
                temp.advance();
            }
            temp.low_byte()
        }
    }
}
