const HIDDEN_SIZE: usize = 768;
const SCALE: i32 = 400;
const QA: i32 = 181;
const QB: i32 = 64;
const QAB: i32 = QA * QB;

pub static NNUE: Network = unsafe { std::mem::transmute(*include_bytes!("../resources/net.bin")) };

#[inline]
/// Activation function
pub fn screlu(x: i16) -> i32 {
    i32::from(x.clamp(0, QA as i16)).pow(2)
}

#[repr(C)]
pub struct Network {
    /// Column-Major `HIDDEN_SIZE x 768` matrix.
    feature_weights: [Accumulator; 768],
    /// Vector with dimension `HIDDEN_SIZE`.
    feature_bias: Accumulator,
    /// Column-Major `1 x (2 * HIDDEN_SIZE)`
    /// matrix, we use it like this to make the
    /// code nicer in `Network::evaluate`.
    output_weights: [Accumulator; 2],
    /// Scalar output bias.
    output_bias: i16,
}

impl Network {
    /// Calculates the output of the network, starting from the already
    /// calculated hidden layer (done efficiently during makemoves).
    pub fn out(boys: &Accumulator, opps: &Accumulator) -> i32 {
        let weights = &NNUE.output_weights;
        let sum = flatten(boys, &weights[0]) + flatten(opps, &weights[1]);
        (sum / QA + i32::from(NNUE.output_bias)) * SCALE / QAB
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C, align(64))]
pub struct Accumulator {
    vals: [i16; HIDDEN_SIZE],
}

impl Accumulator {
    pub fn update<const ADD: bool>(&mut self, idx: usize) {
        assert!(idx < 768);
        for (i, d) in self.vals.iter_mut().zip(&NNUE.feature_weights[idx].vals) {
            if ADD {
                *i += *d
            } else {
                *i -= *d
            }
        }
    }
}

impl Default for Accumulator {
    fn default() -> Self {
        NNUE.feature_bias
    }
}

#[inline]
fn flatten(acc: &Accumulator, weights: &Accumulator) -> i32 {
    let mut sum = 0;

    for (&x, &w) in acc.vals.iter().zip(&weights.vals) {
        sum += screlu(x) * i32::from(w);
    }

    sum
}
