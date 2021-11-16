use dusk_jubjub::{JubJubAffine, JubJubExtended};
use dusk_bls12_381::BlsScalar;
//use lazy_static::lazy_static;


/// The generators (for each segment) used in all Pedersen commitments.
pub const PEDERSEN_HASH_GENERATORS: &[JubJubAffine] = &[
    JubJubAffine::from_raw_unchecked(
        BlsScalar::from_raw([
            0x1010503570c3ebf6,
            0x5c22a82a281c9181,
            0x98ba470b0d28801b,
            0x113de62be6e0d323,
        ]),
        BlsScalar::from_raw([
            0xf031edff274efb14,
            0x2ba3032d7064d633,
            0x15cea14bc9f6b04b,
            0x5059678472abb6ae,
        ]),
    ),
    JubJubAffine::from_raw_unchecked(
        BlsScalar::from_raw([
            0xb9efa2cb80331936,
            0x0a0df10182a290fd,
            0xfc7cbea3c311f67f,
            0x08c02a4c57f7f2cf,
        ]),
        BlsScalar::from_raw([
            0xdaf19ac3ab182662,
            0xec376560c925452d,
            0x4dc07857131f22a0,
            0x2e560a50271fd3fc,
        ]),
    ),
    JubJubAffine::from_raw_unchecked(
        BlsScalar::from_raw([
            0xc93573b98709291e,
            0xdf0694e57c6cbc03,
            0x413bc3c44e7aabe0,
            0x210f22d61b65767d,
        ]),
        BlsScalar::from_raw([
            0x4781e2656b1ddaad,
            0xc6262ed423179659,
            0xfb33884c42727482,
            0x3f46b3371cff7474,
        ]),
    ),
    JubJubAffine::from_raw_unchecked(
        BlsScalar::from_raw([
            0xcf0bc7224a63d094,
            0x2bcc52dbba0ebf3a,
            0xa02f0d3f7aad771d,
            0x274e99b16d4af911,
        ]),
        BlsScalar::from_raw([
            0xe82e9061620a1df4,
            0xfd0153cfe15ec653,
            0x6b15ec6e59478694,
            0x31f5e34f0804a874,
        ]),
    ),
    JubJubAffine::from_raw_unchecked(
        BlsScalar::from_raw([
            0xc64e25ca51961b53,
            0x7058160b9afaafaf,
            0x50aa77ad2f57d2f7,
            0x3ca8b98873e5d19e,
        ]),
        BlsScalar::from_raw([
            0x9dab539b32327842,
            0x5eb152c4606beb7e,
            0x238af7c9376608d6,
            0x10609ce821a5a292,
        ]),
    ),
    JubJubAffine::from_raw_unchecked(
        BlsScalar::from_raw([
            0xf0ef2a816469118e,
            0x5bdd5c30d83781f0,
            0xdb3ff866eaf1bc85,
            0x1ab3fe2ac6b3ff8a,
        ]),
        BlsScalar::from_raw([
            0xe7c079b4e48233f5,
            0xa6b5863148627619,
            0xd5681f2f5c740d19,
            0x2031e442c4af8277,
        ]),
    ),
];

/// The maximum number of chunks per segment of the Pedersen hash.
pub const PEDERSEN_HASH_CHUNKS_PER_GENERATOR: usize = 63;

/// The window size for exponentiation of Pedersen hash generators outside the circuit.
pub const PEDERSEN_HASH_EXP_WINDOW_SIZE: u32 = 8;

lazy_static! {
    /// The exp table for [`PEDERSEN_HASH_GENERATORS`].
    pub static ref PEDERSEN_HASH_EXP_TABLE: Vec<Vec<Vec<JubJubExtended>>> =
        generate_pedersen_hash_exp_table();
}

// The number of bits needed to represent the modulus, from zkcrypto/jubjub
const MODULUS_BITS: u32 = 252;
const NUM_BITS: u32 = MODULUS_BITS;

/// Creates the exp table for the Pedersen hash generators.
fn generate_pedersen_hash_exp_table() -> Vec<Vec<Vec<JubJubExtended>>> {
    let window = PEDERSEN_HASH_EXP_WINDOW_SIZE;

    PEDERSEN_HASH_GENERATORS
        .iter()
        .cloned()
        .map(|g_aff| {
            let mut tables = vec![];
            let mut g = JubJubExtended::from(g_aff);

            let mut num_bits = 0;
            while num_bits <= NUM_BITS {
                let mut table = Vec::with_capacity(1 << window);
                let mut base = JubJubExtended::identity();

                for _ in 0..(1 << window) {
                    table.push(base.clone());
                    base += g;
                }

                tables.push(table);
                num_bits += window;

                for _ in 0..window {
                    g = g.double();
                }
            }

            tables
        })
        .collect()
}