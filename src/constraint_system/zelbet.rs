// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![allow(clippy::too_many_arguments)]

use crate::constraint_system::StandardComposer;
use crate::constraint_system::Variable;
use crate::plookup::table::hash_tables::DECOMPOSITION_S_I;
use crate::plookup::table::hash_tables::INVERSES_S_I;
use crate::plookup::table::hash_tables::{BLS_SCALAR_REAL, SBOX_U256};
use bigint::U256 as u256;
use dusk_bls12_381::BlsScalar;

impl StandardComposer {
    /// Gadget that conducts the bar decomposition, returning the 27-entry
    /// breakdown and adding relevant gates. The input and output variables
    /// are all in Montgomery form, but non-Montgomery form is used within.
    /// [x_27, ..., x_2, x_1] & note that s_i_decomposition should be input
    /// in Montgomery form
    pub fn decomposition_gadget(
        &mut self,
        x: Variable,
        s_i_decomposition: [Variable; 27],
    ) -> [Variable; 27] {
        let mut nibbles = [x; 27];
        // Reduced form needed for the modular operations
        let reduced_input = self.variables[&x].reduce();
        let mut intermediate = u256(reduced_input.0);
        let mut remainder = u256::zero();

        (0..27).for_each(|k| {
            match k < 26 {
                true => {
                    remainder = intermediate % u256(DECOMPOSITION_S_I[k].0);
                    let intermediate_scalar: BlsScalar =
                        BlsScalar((intermediate - remainder).0) * INVERSES_S_I[k];
                    intermediate = u256(intermediate_scalar.0);
                }
                false => remainder = intermediate,
            }

            nibbles[k] = self.add_input(BlsScalar::from_raw(remainder.0));
        });

        // x' = x_1 * s_2 + x_2, this is the start of the composition
        let mut acc = self.big_mul(
            BlsScalar::one(),
            nibbles[26],
            s_i_decomposition[25],
            Some((BlsScalar::one(), nibbles[25])),
            BlsScalar::zero(),
            BlsScalar::zero(),
        );

        (1..26).for_each(|k| {
            acc = self.big_mul(
                BlsScalar::one(),
                acc,
                s_i_decomposition[25 - k],
                Some((BlsScalar::one(), nibbles[25 - k])),
                BlsScalar::zero(),
                BlsScalar::zero(),
            );
        });

        self.constrain_to_constant(acc, self.variables[&x], BlsScalar::zero());

        nibbles
    }

    /// S-box using hash tables, and outputs constraints c_i, z_i and a boolean
    /// counter to help determine the c_i. (y_i, c_i, conditional, z_i)
    pub fn s_box_and_constraints(
        &mut self,
        input: Variable,
        counter: u64,
        conditional: bool,
        one: Variable,
        two: Variable,
    ) -> (Variable, Variable, bool, Variable) {
        // Need to convert input scalar value to non-Montgomery
        // to allow size comparison
        let value = self.variables[&input].reduce();
        let mut y_i = input;
        let mut c_i = one;
        let mut conditional_new = conditional;
        let mut z_i = self.zero_var;
        if value.0[0] < 659 {
            y_i = self.add_input(BlsScalar::from_raw(SBOX_U256[value.0[0] as usize].0));
            conditional_new = true;
        } else {
            y_i = input;
            z_i = one;
            if value.0[0] > BLS_SCALAR_REAL[27 - counter as usize].0[0] {
                c_i = two;
                conditional_new = true
            } else if value.0[0] == BLS_SCALAR_REAL[27 - counter as usize].0[0] {
                if conditional == true {
                    c_i = two;
                    conditional_new = true
                } else {
                    c_i = self.zero_var
                }
            }
        }

        let scaled_z_i = self.add_input(BlsScalar::from(counter) * self.variables[&z_i]);
        self.plookup_gate(input, scaled_z_i, y_i, Some(c_i), BlsScalar::zero());

        (y_i, c_i, conditional_new, z_i)
    }
}

#[cfg(test)]
mod tests {
    use super::super::helper::*;
    use super::*;
    use crate::plookup::table::hash_tables::constants::S_I_DECOMPOSITION_MONTGOMERY;
    use crate::plookup::table::hash_tables::DECOMPOSITION_S_I;
    use crate::plookup::PlookupTable4Arity;
    use dusk_bls12_381::BlsScalar;

    #[test]
    fn test_s_box_and_constraints() {
        let res = gadget_tester(
            |composer| {
                let hash_table = PlookupTable4Arity::create_hash_table();
                composer.append_lookup_table(&hash_table);
                let seven_hundred = composer.add_input(BlsScalar::from(700));
                let one = composer.add_input(BlsScalar::one());
                let two = composer.add_input(BlsScalar::from(2));
                let prime = composer.add_input(BlsScalar::from(659));
                let counter: u64 = 1;
                let counter2: u64 = 2;
                let conditional = true;
                let output_700 =
                    composer.s_box_and_constraints(seven_hundred, counter2, conditional, one, two);
                let output_one =
                    composer.s_box_and_constraints(one, counter, conditional, one, two);
                let output_prime =
                    composer.s_box_and_constraints(prime, counter2, conditional, one, two);
                let output_prime_false =
                    composer.s_box_and_constraints(prime, counter, false, one, two);

                // Check that the s-box works as expected
                composer.constrain_to_constant(
                    output_700.0,
                    BlsScalar::from_raw([700, 0, 0, 0]),
                    BlsScalar::zero(),
                );
                composer.constrain_to_constant(
                    output_one.0,
                    BlsScalar::from_raw([187, 0, 0, 0]),
                    BlsScalar::zero(),
                );
                composer.constrain_to_constant(
                    output_prime.0,
                    BlsScalar::from_raw([659, 0, 0, 0]),
                    BlsScalar::zero(),
                );

                (0..1100).for_each(|k| {
                    composer.plookup_gate(prime, one, prime, Some(one), BlsScalar::zero());
                });

                // Check that the c_i are output as expected
                assert_eq!(composer.variables[&output_700.1], BlsScalar::from(2));
                assert_eq!(composer.variables[&output_one.1], BlsScalar::from(1));
                assert_eq!(composer.variables[&output_prime.1], BlsScalar::from(1));
                assert_eq!(
                    composer.variables[&output_prime_false.1],
                    BlsScalar::from(1)
                );

                // Check that the counter is output correctly
                assert!(output_700.2);
                assert!(output_one.2);
                assert!(output_prime.2);
                assert!(!output_prime_false.2);

                // Check that z_i is output correctly
                assert_eq!(composer.variables[&output_700.3], BlsScalar::from(1));
                assert_eq!(composer.variables[&output_one.3], BlsScalar::from(0));
                assert_eq!(composer.variables[&output_prime.3], BlsScalar::from(1));
                assert_eq!(
                    composer.variables[&output_prime_false.3],
                    BlsScalar::from(1)
                );
            },
            4000,
        );
        assert!(res.is_ok());
    }

    #[test]
    fn test_s_box_and_constraints_fails() {
        let res = gadget_tester(
            |composer| {
                let hash_table = PlookupTable4Arity::create_hash_table();
                composer.append_lookup_table(&hash_table);
                let one_hundred = composer.add_input(BlsScalar::from(100));
                let two = composer.add_input(BlsScalar::from(2));
                let counter: u64 = 1;
                let conditional = true;
                let output = composer.s_box_and_constraints(
                    one_hundred,
                    counter,
                    conditional,
                    one_hundred,
                    two,
                );
                composer.constrain_to_constant(
                    output.0,
                    BlsScalar::from_raw([200, 0, 0, 0]),
                    BlsScalar::zero(),
                );
                composer.constrain_to_constant(
                    output.0,
                    BlsScalar::from_raw([200, 0, 0, 0]),
                    BlsScalar::zero(),
                );
                composer.constrain_to_constant(
                    output.0,
                    BlsScalar::from_raw([200, 0, 0, 0]),
                    BlsScalar::zero(),
                );

                let prime = composer.add_input(BlsScalar::from(659));
                (0..1100).for_each(|k| {
                    composer.plookup_gate(
                        prime,
                        one_hundred,
                        prime,
                        Some(one_hundred),
                        BlsScalar::zero(),
                    );
                });
            },
            2000,
        );
        assert!(res.is_err());
    }

    #[test]
    fn test_decomposition() {
        let res = gadget_tester(
            |composer| {
                let one = composer.add_input(BlsScalar::one());
                let mut s_i_decomposition = [one; 27];
                (0..27).for_each(|k| {
                    s_i_decomposition[k] = composer.add_input(S_I_DECOMPOSITION_MONTGOMERY[k]);
                });
                let output = composer.decomposition_gadget(one, s_i_decomposition);
                (1..27).for_each(|k| {
                    composer.constrain_to_constant(output[k], BlsScalar::zero(), BlsScalar::zero());
                });
                // Check x_27 = 1, bearing in mind that x_1 is not in Montgomery form
                composer.constrain_to_constant(output[0], BlsScalar::one(), BlsScalar::zero());

                let minus_three = composer.add_input(-BlsScalar::from(3));
                let output2 = composer.decomposition_gadget(minus_three, s_i_decomposition);
                // Expetced output derived from out of circuit version
                let expected_output = [
                    658, 660, 673, 663, 674, 682, 687, 683, 669, 684, 672, 666, 680, 662, 686, 668,
                    661, 678, 692, 686, 689, 660, 690, 687, 683, 674, 678, 658, 660, 673, 663, 674,
                    682, 687, 683, 669, 684, 672, 666, 680, 662, 686, 668, 661, 678, 692, 686, 689,
                    660, 690, 687, 683, 674, 678, 658, 660, 673, 663, 674, 682, 687, 683, 669, 684,
                    672, 666, 680, 662, 686, 668, 661, 678, 692, 686, 689, 660, 690, 687, 683, 674,
                    678,
                ];
                (0..27).for_each(|k| {
                    composer.constrain_to_constant(
                        output2[k],
                        BlsScalar::from(expected_output[k]),
                        BlsScalar::zero(),
                    );
                })
            },
            500,
        );
        assert!(res.is_ok());
    }
}
