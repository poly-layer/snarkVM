// Copyright (C) 2019-2021 Aleo Systems Inc.
// This file is part of the snarkVM library.

// The snarkVM library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The snarkVM library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the snarkVM library. If not, see <https://www.gnu.org/licenses/>.

use crate::{
    algorithms::crypto_hash::{CryptographicSpongeVar, PoseidonSpongeGadget},
    AllocGadget,
    Boolean,
    ConditionalEqGadget,
    EncryptionGadget,
    EqGadget,
    FieldGadget,
    FpGadget,
    GroupGadget,
    Integer,
    ToBytesGadget,
    UInt8,
};
use itertools::Itertools;
use snarkvm_algorithms::{
    crypto_hash::PoseidonDefaultParametersField,
    encryption::{ECIESPoseidonEncryption, ECIESPoseidonPublicKey},
};
use snarkvm_curves::{
    templates::twisted_edwards_extended::{Affine as TEAffine, Projective as TEProjective},
    AffineCurve,
    Group,
    ProjectiveCurve,
    TwistedEdwardsParameters,
};
use snarkvm_fields::{FieldParameters, PrimeField};
use snarkvm_r1cs::{ConstraintSystem, SynthesisError};
use snarkvm_utilities::{borrow::Borrow, to_bytes_le, ToBytes};
use std::marker::PhantomData;

type TEAffineGadget<TE, F> = crate::curves::templates::twisted_edwards::AffineGadget<TE, F, FpGadget<F>>;

/// ECIES encryption private key gadget
#[derive(Derivative)]
#[derivative(
    Clone(bound = "TE: TwistedEdwardsParameters<BaseField = F>, F: PrimeField"),
    PartialEq(bound = "TE: TwistedEdwardsParameters<BaseField = F>, F: PrimeField"),
    Eq(bound = "TE: TwistedEdwardsParameters<BaseField = F>, F: PrimeField"),
    Debug(bound = "TE: TwistedEdwardsParameters<BaseField = F>, F: PrimeField")
)]
pub struct ECIESPoseidonEncryptionPrivateKeyGadget<TE: TwistedEdwardsParameters<BaseField = F>, F: PrimeField>(
    pub Vec<UInt8>,
    PhantomData<TE>,
    PhantomData<F>,
);

impl<TE: TwistedEdwardsParameters<BaseField = F>, F: PrimeField> AllocGadget<TE::ScalarField, F>
    for ECIESPoseidonEncryptionPrivateKeyGadget<TE, F>
{
    fn alloc_constant<
        Fn: FnOnce() -> Result<T, SynthesisError>,
        T: Borrow<TE::ScalarField>,
        CS: ConstraintSystem<F>,
    >(
        _cs: CS,
        value_gen: Fn,
    ) -> Result<Self, SynthesisError> {
        let private_key = to_bytes_le![value_gen()?.borrow()].unwrap();
        Ok(ECIESPoseidonEncryptionPrivateKeyGadget(
            UInt8::constant_vec(&private_key),
            PhantomData,
            PhantomData,
        ))
    }

    fn alloc<Fn: FnOnce() -> Result<T, SynthesisError>, T: Borrow<TE::ScalarField>, CS: ConstraintSystem<F>>(
        cs: CS,
        value_gen: Fn,
    ) -> Result<Self, SynthesisError> {
        let private_key = to_bytes_le![value_gen()?.borrow()].unwrap();
        Ok(ECIESPoseidonEncryptionPrivateKeyGadget(
            UInt8::alloc_vec(cs, &private_key)?,
            PhantomData,
            PhantomData,
        ))
    }

    fn alloc_input<Fn: FnOnce() -> Result<T, SynthesisError>, T: Borrow<TE::ScalarField>, CS: ConstraintSystem<F>>(
        cs: CS,
        value_gen: Fn,
    ) -> Result<Self, SynthesisError> {
        let private_key = to_bytes_le![value_gen()?.borrow()].unwrap();
        Ok(ECIESPoseidonEncryptionPrivateKeyGadget(
            UInt8::alloc_input_vec_le(cs, &private_key)?,
            PhantomData,
            PhantomData,
        ))
    }
}

impl<TE: TwistedEdwardsParameters<BaseField = F>, F: PrimeField> ToBytesGadget<F>
    for ECIESPoseidonEncryptionPrivateKeyGadget<TE, F>
{
    fn to_bytes<CS: ConstraintSystem<F>>(&self, mut cs: CS) -> Result<Vec<UInt8>, SynthesisError> {
        self.0.to_bytes(&mut cs.ns(|| "to_bytes"))
    }

    fn to_bytes_strict<CS: ConstraintSystem<F>>(&self, mut cs: CS) -> Result<Vec<UInt8>, SynthesisError> {
        self.0.to_bytes_strict(&mut cs.ns(|| "to_bytes_strict"))
    }
}

/// ECIES encryption randomness gadget
#[derive(Derivative)]
#[derivative(
    Clone(bound = "TE: TwistedEdwardsParameters"),
    PartialEq(bound = "TE: TwistedEdwardsParameters"),
    Eq(bound = "TE: TwistedEdwardsParameters"),
    Debug(bound = "TE: TwistedEdwardsParameters")
)]
pub struct ECIESPoseidonEncryptionRandomnessGadget<TE: TwistedEdwardsParameters>(pub Vec<UInt8>, PhantomData<TE>);

impl<TE: TwistedEdwardsParameters> AllocGadget<TE::ScalarField, TE::BaseField>
    for ECIESPoseidonEncryptionRandomnessGadget<TE>
where
    TE::BaseField: PrimeField,
{
    fn alloc_constant<
        Fn: FnOnce() -> Result<T, SynthesisError>,
        T: Borrow<TE::ScalarField>,
        CS: ConstraintSystem<TE::BaseField>,
    >(
        _cs: CS,
        value_gen: Fn,
    ) -> Result<Self, SynthesisError> {
        let randomness = to_bytes_le![value_gen()?.borrow()].unwrap();
        Ok(ECIESPoseidonEncryptionRandomnessGadget(
            UInt8::constant_vec(&randomness),
            PhantomData,
        ))
    }

    fn alloc<
        Fn: FnOnce() -> Result<T, SynthesisError>,
        T: Borrow<TE::ScalarField>,
        CS: ConstraintSystem<TE::BaseField>,
    >(
        cs: CS,
        value_gen: Fn,
    ) -> Result<Self, SynthesisError> {
        let randomness = to_bytes_le![value_gen()?.borrow()].unwrap();
        Ok(ECIESPoseidonEncryptionRandomnessGadget(
            UInt8::alloc_vec(cs, &randomness)?,
            PhantomData,
        ))
    }

    fn alloc_input<
        Fn: FnOnce() -> Result<T, SynthesisError>,
        T: Borrow<TE::ScalarField>,
        CS: ConstraintSystem<TE::BaseField>,
    >(
        cs: CS,
        value_gen: Fn,
    ) -> Result<Self, SynthesisError> {
        let randomness = to_bytes_le![value_gen()?.borrow()].unwrap();
        Ok(ECIESPoseidonEncryptionRandomnessGadget(
            UInt8::alloc_input_vec_le(cs, &randomness)?,
            PhantomData,
        ))
    }
}

/// ECIES Poseidon encryption gadget
#[derive(Derivative)]
#[derivative(
    Clone(bound = "TE: TwistedEdwardsParameters<BaseField = F>, F: PrimeField"),
    PartialEq(bound = "TE: TwistedEdwardsParameters<BaseField = F>, F: PrimeField"),
    Eq(bound = "TE: TwistedEdwardsParameters<BaseField = F>, F: PrimeField"),
    Debug(bound = "TE: TwistedEdwardsParameters<BaseField = F>, F: PrimeField")
)]
pub struct ECIESPoseidonEncryptionGadget<TE: TwistedEdwardsParameters<BaseField = F>, F: PrimeField> {
    encryption: ECIESPoseidonEncryption<TE>,
    f_phantom: PhantomData<F>,
}

impl<TE: TwistedEdwardsParameters<BaseField = F>, F: PrimeField> AllocGadget<ECIESPoseidonEncryption<TE>, F>
    for ECIESPoseidonEncryptionGadget<TE, F>
{
    fn alloc_constant<
        Fn: FnOnce() -> Result<T, SynthesisError>,
        T: Borrow<ECIESPoseidonEncryption<TE>>,
        CS: ConstraintSystem<F>,
    >(
        _cs: CS,
        value_gen: Fn,
    ) -> Result<Self, SynthesisError> {
        Ok(Self {
            encryption: (*value_gen()?.borrow()).clone(),
            f_phantom: PhantomData,
        })
    }

    fn alloc<
        Fn: FnOnce() -> Result<T, SynthesisError>,
        T: Borrow<ECIESPoseidonEncryption<TE>>,
        CS: ConstraintSystem<F>,
    >(
        _cs: CS,
        _value_gen: Fn,
    ) -> Result<Self, SynthesisError> {
        unimplemented!()
    }

    fn alloc_input<
        Fn: FnOnce() -> Result<T, SynthesisError>,
        T: Borrow<ECIESPoseidonEncryption<TE>>,
        CS: ConstraintSystem<F>,
    >(
        _cs: CS,
        _value_gen: Fn,
    ) -> Result<Self, SynthesisError> {
        unimplemented!()
    }
}

/// ECIES Poseidon public key gadget
#[derive(Derivative)]
#[derivative(
    Clone(bound = "TE: TwistedEdwardsParameters<BaseField = F>, F: PrimeField"),
    PartialEq(bound = "TE: TwistedEdwardsParameters<BaseField = F>, F: PrimeField"),
    Eq(bound = "TE: TwistedEdwardsParameters<BaseField = F>, F: PrimeField"),
    Debug(bound = "TE: TwistedEdwardsParameters<BaseField = F>, F: PrimeField")
)]
pub struct ECIESPoseidonEncryptionPublicKeyGadget<TE: TwistedEdwardsParameters<BaseField = F>, F: PrimeField>(
    TEAffineGadget<TE, F>,
);

impl<TE: TwistedEdwardsParameters<BaseField = F>, F: PrimeField> AllocGadget<ECIESPoseidonPublicKey<TE>, F>
    for ECIESPoseidonEncryptionPublicKeyGadget<TE, F>
where
    TEAffineGadget<TE, F>: GroupGadget<TEAffine<TE>, F>,
{
    fn alloc_constant<
        Fn: FnOnce() -> Result<T, SynthesisError>,
        T: Borrow<ECIESPoseidonPublicKey<TE>>,
        CS: ConstraintSystem<TE::BaseField>,
    >(
        cs: CS,
        value_gen: Fn,
    ) -> Result<Self, SynthesisError> {
        Ok(Self {
            0: TEAffineGadget::<TE, F>::alloc_constant(cs, || Ok(value_gen()?.borrow().0.clone()))?,
        })
    }

    fn alloc<
        Fn: FnOnce() -> Result<T, SynthesisError>,
        T: Borrow<ECIESPoseidonPublicKey<TE>>,
        CS: ConstraintSystem<TE::BaseField>,
    >(
        cs: CS,
        value_gen: Fn,
    ) -> Result<Self, SynthesisError> {
        Ok(Self {
            0: TEAffineGadget::<TE, F>::alloc(cs, || Ok(value_gen()?.borrow().0.clone()))?,
        })
    }

    fn alloc_input<
        Fn: FnOnce() -> Result<T, SynthesisError>,
        T: Borrow<ECIESPoseidonPublicKey<TE>>,
        CS: ConstraintSystem<TE::BaseField>,
    >(
        cs: CS,
        value_gen: Fn,
    ) -> Result<Self, SynthesisError> {
        Ok(Self {
            0: TEAffineGadget::<TE, F>::alloc_input(cs, || Ok(value_gen()?.borrow().0.clone()))?,
        })
    }
}

impl<TE: TwistedEdwardsParameters<BaseField = F>, F: PrimeField> ConditionalEqGadget<F>
    for ECIESPoseidonEncryptionPublicKeyGadget<TE, F>
{
    fn conditional_enforce_equal<CS: ConstraintSystem<F>>(
        &self,
        cs: CS,
        other: &Self,
        condition: &Boolean,
    ) -> Result<(), SynthesisError> {
        self.0.conditional_enforce_equal(cs, &other.0, condition)
    }

    fn cost() -> usize {
        <TEAffineGadget<TE, F> as ConditionalEqGadget<F>>::cost()
    }
}

impl<TE: TwistedEdwardsParameters<BaseField = F>, F: PrimeField> ToBytesGadget<F>
    for ECIESPoseidonEncryptionPublicKeyGadget<TE, F>
{
    fn to_bytes<CS: ConstraintSystem<F>>(&self, cs: CS) -> Result<Vec<UInt8>, SynthesisError> {
        self.0.x.to_bytes(cs)
    }

    fn to_bytes_strict<CS: ConstraintSystem<F>>(&self, cs: CS) -> Result<Vec<UInt8>, SynthesisError> {
        self.0.x.to_bytes_strict(cs)
    }
}

impl<TE: TwistedEdwardsParameters<BaseField = F>, F: PrimeField> EqGadget<F>
    for ECIESPoseidonEncryptionPublicKeyGadget<TE, F>
{
}

impl<TE: TwistedEdwardsParameters<BaseField = F>, F: PrimeField + PoseidonDefaultParametersField>
    EncryptionGadget<ECIESPoseidonEncryption<TE>, F> for ECIESPoseidonEncryptionGadget<TE, F>
{
    type PrivateKeyGadget = ECIESPoseidonEncryptionPrivateKeyGadget<TE, F>;
    type PublicKeyGadget = ECIESPoseidonEncryptionPublicKeyGadget<TE, F>;
    type RandomnessGadget = ECIESPoseidonEncryptionRandomnessGadget<TE>;

    fn check_public_key_gadget<CS: ConstraintSystem<TE::BaseField>>(
        &self,
        mut cs: CS,
        private_key: &Self::PrivateKeyGadget,
    ) -> Result<Self::PublicKeyGadget, SynthesisError> {
        let private_key_bits = private_key.0.iter().flat_map(|b| b.to_bits_le()).collect::<Vec<_>>();
        let mut public_key = <TEAffineGadget<TE, F> as GroupGadget<TEAffine<TE>, F>>::zero(cs.ns(|| "zero"))?;

        let num_powers = private_key_bits.len();

        let generator_powers: Vec<TEAffine<TE>> = {
            let mut generator_powers = Vec::new();
            let mut generator = self.encryption.generator.into_projective();
            for _ in 0..num_powers {
                generator_powers.push(generator.clone());
                generator.double_in_place();
            }
            TEProjective::<TE>::batch_normalization(&mut generator_powers);
            generator_powers.into_iter().map(|v| v.into()).collect()
        };

        public_key.scalar_multiplication(
            cs.ns(|| "check_public_key_gadget"),
            private_key_bits.iter().zip_eq(&generator_powers),
        )?;

        Ok(ECIESPoseidonEncryptionPublicKeyGadget::<TE, F> { 0: public_key })
    }

    fn check_encryption_gadget<CS: ConstraintSystem<F>>(
        &self,
        mut cs: CS,
        randomness: &Self::RandomnessGadget,
        public_key: &Self::PublicKeyGadget,
        message: &[UInt8],
    ) -> Result<Vec<UInt8>, SynthesisError> {
        let affine_zero: TEAffineGadget<TE, F> =
            <TEAffineGadget<TE, F> as GroupGadget<TEAffine<TE>, F>>::zero(cs.ns(|| "affine zero")).unwrap();
        let group_zero = FpGadget::<TE::BaseField>::zero(cs.ns(|| "field zero"))?;

        // Compute the ECDH value.
        let randomness_bits = randomness.0.iter().flat_map(|b| b.to_bits_le()).collect::<Vec<_>>();
        let ecdh_value = <TEAffineGadget<TE, F> as GroupGadget<TEAffine<TE>, F>>::mul_bits(
            &public_key.0,
            cs.ns(|| "compute_ecdh_value"),
            &affine_zero,
            randomness_bits.iter().copied(),
        )?;

        // Prepare the sponge.
        let params =
            <TE::BaseField as PoseidonDefaultParametersField>::get_default_poseidon_parameters(4, false).unwrap();
        let mut sponge = PoseidonSpongeGadget::<TE::BaseField>::new(cs.ns(|| "sponge"), &params);
        sponge.absorb(cs.ns(|| "absorb"), [ecdh_value.x].iter())?;

        // Convert the message into bits.
        let mut bits = Vec::with_capacity(message.len() * 8 + 1);
        for byte in message.iter() {
            bits.extend_from_slice(&byte.to_bits_le());
        }
        bits.push(Boolean::Constant(true));
        // The last bit indicates the end of the actual data, which is used in decoding to
        // make sure that the length is correct.

        // Pack the bits into field elements.
        let capacity = <<TE::BaseField as PrimeField>::Parameters as FieldParameters>::CAPACITY as usize;
        let mut res = Vec::with_capacity((bits.len() + capacity - 1) / capacity);
        for (i, chunk) in bits.chunks(capacity).enumerate() {
            let mut sum = group_zero.clone();

            let mut cur = TE::BaseField::one();
            for (j, bit) in chunk.iter().enumerate() {
                let add =
                    FpGadget::from_boolean(cs.ns(|| format!("convert a bit to a field element {} {}", i, j)), *bit)?
                        .mul_by_constant(cs.ns(|| format!("multiply by the shift {} {}", i, j)), &cur)?;
                sum.add_in_place(
                    cs.ns(|| format!("assemble the bit result into field elements {} {}", i, j)),
                    &add,
                )?;

                cur.double_in_place();
            }

            res.push(sum);
        }

        // Obtain random field elements from Poseidon.
        let sponge_field_elements =
            sponge.squeeze_field_elements(cs.ns(|| "squeeze for random elements"), res.len())?;

        // Add the random field elements to the packed bits.
        for (i, sponge_field_element) in sponge_field_elements.iter().enumerate() {
            res[i].add_in_place(
                cs.ns(|| format!("add the sponge field element {}", i)),
                sponge_field_element,
            )?;
        }

        let res_bytes = res.to_bytes(cs.ns(|| "convert the masked results into bytes"))?;

        // Put the bytes of the x coordinate of the randomness group element
        // into the beginning of the ciphertext.
        let generator_gadget = TEAffineGadget::<TE, F>::alloc_constant(cs.ns(|| "alloc generator"), || {
            Ok(self.encryption.generator.clone())
        })?;
        let randomness_elem = <TEAffineGadget<TE, F> as GroupGadget<TEAffine<TE>, F>>::mul_bits(
            &generator_gadget,
            cs.ns(|| "compute the randomness element"),
            &affine_zero,
            randomness_bits.iter().copied(),
        )?;
        let random_elem_bytes = randomness_elem
            .x
            .to_bytes(cs.ns(|| "convert the randomness element to bytes"))?;

        Ok([random_elem_bytes, res_bytes].concat())
    }
}
