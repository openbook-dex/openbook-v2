<!-- Copyright © 2018–2021 Trevor Spiteri -->

<!-- Copying and distribution of this file, with or without
modification, are permitted in any medium without royalty provided the
copyright notice and this notice are preserved. This file is offered
as-is, without any warranty. -->

Version 1.11.0 (2021-11-24)
===========================

  * The following methods were added to all fixed-point numbers, to the
    [`Fixed`][tf-1-11] trait, and to the [`Wrapping`][w-1-11] and
    [`Unwrapped`][u-1-11] wrappers:
      * [`lerp`][f-l-1-11]
      * [`inv_lerp`][f-il-1-11]
  * The following methods were added to all fixed-point numbers and to the
    [`Fixed`][tf-1-11] trait:
      * [`checked_lerp`][f-cl-1-11], [`saturating_lerp`][f-sl-1-11],
        [`wrapping_lerp`][f-wl-1-11], [`unwrapped_lerp`][f-ul-1-11],
        [`overflowing_lerp`][f-ol-1-11]
      * [`checked_inv_lerp`][f-cil-1-11], [`saturating_inv_lerp`][f-sil-1-11],
        [`wrapping_inv_lerp`][f-wil-1-11], [`unwrapped_inv_lerp`][f-uil-1-11],
        [`overflowing_inv_lerp`][f-oil-1-11]
  * The [*typenum* crate] dependency was updated to [version
    1.14][typenum-1-14].
  * The [`LeEqU8`][leu8-1-11], [`LeEqU16`][leu16-1-11], [`LeEqU32`][leu32-1-11],
    [`LeEqU64`][leu64-1-11] and [`LeEqU128`][leu128-1-11] traits no longer have
    a direct `'static` constraint, as it is a constraint of their supertrait
    [`Unsigned`][uns-1-11] since typenum [version 1.14][typenum-1-14]. This
    fixes a potential compatibility issue introduced in version 1.9.0.
  * An experimental feature was added to enable serialization using the [*borsh*
    crate] ([merge request 9]).

[f-cil-1-11]: https://docs.rs/fixed/~1.11/fixed/struct.FixedI32.html#method.checked_inv_lerp
[f-cl-1-11]: https://docs.rs/fixed/~1.11/fixed/struct.FixedI32.html#method.checked_lerp
[f-il-1-11]: https://docs.rs/fixed/~1.11/fixed/struct.FixedI32.html#method.inv_lerp
[f-l-1-11]: https://docs.rs/fixed/~1.11/fixed/struct.FixedI32.html#method.lerp
[f-oil-1-11]: https://docs.rs/fixed/~1.11/fixed/struct.FixedI32.html#method.overflowing_inv_lerp
[f-ol-1-11]: https://docs.rs/fixed/~1.11/fixed/struct.FixedI32.html#method.overflowing_lerp
[f-sil-1-11]: https://docs.rs/fixed/~1.11/fixed/struct.FixedI32.html#method.saturating_inv_lerp
[f-sl-1-11]: https://docs.rs/fixed/~1.11/fixed/struct.FixedI32.html#method.saturating_lerp
[f-uil-1-11]: https://docs.rs/fixed/~1.11/fixed/struct.FixedI32.html#method.unwrapped_inv_lerp
[f-ul-1-11]: https://docs.rs/fixed/~1.11/fixed/struct.FixedI32.html#method.unwrapped_lerp
[f-wil-1-11]: https://docs.rs/fixed/~1.11/fixed/struct.FixedI32.html#method.wrapping_inv_lerp
[f-wl-1-11]: https://docs.rs/fixed/~1.11/fixed/struct.FixedI32.html#method.wrapping_lerp
[leu128-1-11]: https://docs.rs/fixed/~1.11/fixed/types/extra/trait.LeEqU128.html
[leu16-1-11]: https://docs.rs/fixed/~1.11/fixed/types/extra/trait.LeEqU16.html
[leu32-1-11]: https://docs.rs/fixed/~1.11/fixed/types/extra/trait.LeEqU32.html
[leu64-1-11]: https://docs.rs/fixed/~1.11/fixed/types/extra/trait.LeEqU64.html
[leu8-1-11]: https://docs.rs/fixed/~1.11/fixed/types/extra/trait.LeEqU8.html
[merge request 9]: https://gitlab.com/tspiteri/fixed/-/merge_requests/9
[tf-1-11]: https://docs.rs/fixed/~1.11/fixed/traits/trait.Fixed.html
[typenum-1-14]: https://docs.rs/typenum/~1.14/typenum/index.html
[u-1-11]: https://docs.rs/fixed/~1.11/fixed/struct.Unwrapped.html
[uns-1-11]: https://docs.rs/fixed/~1.11/fixed/types/extra/trait.Unsigned.html
[w-1-11]: https://docs.rs/fixed/~1.11/fixed/struct.Wrapping.html

Version 1.10.0 (2021-08-23)
===========================

  * The crate now requires rustc version 1.53.0 or later.
  * <code>{[Div][`Div`],[DivAssign][`DivAssign`],[Rem][`Rem`],[RemAssign][`RemAssign`]}\<[NonZeroU32][`NonZeroU32`]></code>
    are now implemented for [`FixedU32`][fu-1-10], and similar for all other
    unsigned fixed-point numbers.
  * <code>{[Rem][`Rem`],[RemAssign][`RemAssign`]}\<[NonZeroI32][`NonZeroI32`]></code>
    are now implemented for [`FixedI32`][fi-1-10], and similar for all other
    signed fixed-point numbers.
  * The new [`arbitrary`][feat-1-10] optional feature was added to implement the
    [`Arbitrary`][a-a-1] trait provided by the [*arbitrary* crate] for all
    fixed-point numbers ([issue 37]).

[a-a-1]: https://docs.rs/arbitrary/^1/arbitrary/trait.Arbitrary.html
[feat-1-10]: https://docs.rs/fixed/~1.10/fixed/index.html#optional-features
[fi-1-10]: https://docs.rs/fixed/~1.10/fixed/struct.FixedI32.html
[fu-1-10]: https://docs.rs/fixed/~1.10/fixed/struct.FixedU32.html
[issue 37]: https://gitlab.com/tspiteri/fixed/-/issues/37

Version 1.9.0 (2021-05-13)
==========================

  * Fixed-point numbers can now be formatted as hexadecimal with [`Debug`]
    similarly to primitive integers, for example formatting with `{:X?}` will
    produce upper-case hexadecimal fixed-point numbers.
  * The following methods were added to all fixed-point numbers, to the
    [`Fixed`][tf-1-9] trait, and to the [`Wrapping`][w-1-9] and
    [`Unwrapped`][u-1-9] wrappers:
      * [`is_zero`][f-iz-1-9]
      * [`dist`][f-d-1-9]
  * The following methods were added to all fixed-point numbers and to the
    [`Fixed`][tf-1-9] trait:
      * [`checked_dist`][f-cd-1-9], [`saturating_dist`][f-sd-1-9],
        [`wrapping_dist`][f-wd-1-9], [`unwrapped_dist`][f-ud-1-9],
        [`overflowing_dist`][f-od-1-9]
  * The [`unsigned_dist`][f-unsd-1-9] method was added to all signed
    fixed-point types and to the [`FixedSigned`][tfs-1-9] trait.
  * The following associated types and provided methods were added to the
    [`Fixed`][tf-1-9] trait:
      * [`Signed`][tf-s-1-9], [`Unsigned`][tf-u-1-9]
      * [`get_signed`][tf-gs-1-9], [`get_unsigned`][tf-gu-1-9]
      * [`get_signed_mut`][tf-gsm-1-9], [`get_unsigned_mut`][tf-gum-1-9]
  * The new trait [`FixedEquiv`][fe-1-9] was added.
  * The following traits from the [*bytemuck* crate] were implemented for all
    fixed-point numbers, added as supertraits to the [`Fixed`][tf-1-9] trait,
    and implemented for the [`Wrapping`][w-1-9] and [`Unwrapped`][u-1-9]
    wrappers ([issue 31]).
      * [`Zeroable`][bm-z-1], [`Pod`][bm-p-1]
      * [`TransparentWrapper`][bm-tw-1]

Compatibility notes
-------------------

  * Now the [`Debug`] implementation for [`Wrapping`][w-1-9] outputs the value
    only without “`Wrapping()`”, and the [`Debug`] implementation for
    [`Unwrapped`][u-1-9] outputs the value only without “`Unwrapped()`”.
  * The [`LeEqU8`][leu8-1-9], [`LeEqU16`][leu16-1-9], [`LeEqU32`][leu32-1-9],
    [`LeEqU64`][leu64-1-9] and [`LeEqU128`][leu128-1-9] traits now have a
    `'static` constraint. This should have no practical side effects, since
    these traits are a convenience feature and already have the
    [`Unsigned`][uns-1-9] marker trait as a supertrait, and the types that
    implement [`Unsigned`][uns-1-9] are `'static`.
  * The [`FixedOptionalFeatures`][fof-1-9] trait was not sealed, which was as an
    oversight. Now it is sealed, and the documentation explicitly states that
    the trait should not be used directly.

[bm-p-1]: https://docs.rs/bytemuck/^1/bytemuck/trait.Pod.html
[bm-tw-1]: https://docs.rs/bytemuck/^1/bytemuck/trait.TransparentWrapper.html
[bm-z-1]: https://docs.rs/bytemuck/^1/bytemuck/trait.Zeroable.html
[f-cd-1-9]: https://docs.rs/fixed/~1.9/fixed/struct.FixedI32.html#method.checked_dist
[f-d-1-9]: https://docs.rs/fixed/~1.9/fixed/struct.FixedI32.html#method.dist
[f-iz-1-9]: https://docs.rs/fixed/~1.9/fixed/struct.FixedI32.html#method.is_zero
[f-od-1-9]: https://docs.rs/fixed/~1.9/fixed/struct.FixedI32.html#method.overflowing_dist
[f-sd-1-9]: https://docs.rs/fixed/~1.9/fixed/struct.FixedI32.html#method.saturating_dist
[f-ud-1-9]: https://docs.rs/fixed/~1.9/fixed/struct.FixedI32.html#method.unwrapped_dist
[f-unsd-1-9]: https://docs.rs/fixed/~1.9/fixed/struct.FixedI32.html#method.unsigned_dist
[f-wd-1-9]: https://docs.rs/fixed/~1.9/fixed/struct.FixedI32.html#method.wrapping_dist
[fe-1-9]: https://docs.rs/fixed/~1.9/fixed/traits/trait.FixedEquiv.html
[fof-1-9]: https://docs.rs/fixed/~1.9/fixed/traits/trait.FixedOptionalFeatures.html
[issue 31]: https://gitlab.com/tspiteri/fixed/-/issues/31
[leu128-1-9]: https://docs.rs/fixed/~1.9/fixed/types/extra/trait.LeEqU128.html
[leu16-1-9]: https://docs.rs/fixed/~1.9/fixed/types/extra/trait.LeEqU16.html
[leu32-1-9]: https://docs.rs/fixed/~1.9/fixed/types/extra/trait.LeEqU32.html
[leu64-1-9]: https://docs.rs/fixed/~1.9/fixed/types/extra/trait.LeEqU64.html
[leu8-1-9]: https://docs.rs/fixed/~1.9/fixed/types/extra/trait.LeEqU8.html
[tf-1-9]: https://docs.rs/fixed/~1.9/fixed/traits/trait.Fixed.html
[tf-gs-1-9]: https://docs.rs/fixed/~1.9/fixed/traits/trait.Fixed.html#method.get_signed
[tf-gsm-1-9]: https://docs.rs/fixed/~1.9/fixed/traits/trait.Fixed.html#method.get_signed_mut
[tf-gu-1-9]: https://docs.rs/fixed/~1.9/fixed/traits/trait.Fixed.html#method.get_unsigned
[tf-gum-1-9]: https://docs.rs/fixed/~1.9/fixed/traits/trait.Fixed.html#method.get_unsigned_mut
[tf-s-1-9]: https://docs.rs/fixed/~1.9/fixed/traits/trait.Fixed.html#associatedtype.Signed
[tf-u-1-9]: https://docs.rs/fixed/~1.9/fixed/traits/trait.Fixed.html#associatedtype.Unsigned
[tfs-1-9]: https://docs.rs/fixed/~1.9/fixed/traits/trait.FixedSigned.html
[u-1-9]: https://docs.rs/fixed/~1.9/fixed/struct.Unwrapped.html
[uns-1-9]: https://docs.rs/fixed/~1.9/fixed/types/extra/trait.Unsigned.html
[w-1-9]: https://docs.rs/fixed/~1.9/fixed/struct.Wrapping.html

Version 1.8.0 (2021-04-20)
==========================

  * The following constants and method were added to all fixed-point numbers, to
    the [`Fixed`][tf-1-8] trait, and to the [`Wrapping`][w-1-8] and
    [`Unwrapped`][u-1-8] wrappers:
      * [`ZERO`][f-z-1-8], [`DELTA`][f-d-1-8]
      * [`mul_acc`][f-ma-1-8]
  * The [`ONE`][f-o-1-8] constant was added to all fixed-point numbers that can
    represent the value 1.
  * The following methods were added to all fixed-point numbers and to the
    [`Fixed`][tf-1-8] trait:
      * [`checked_mul_acc`][f-cma-1-8], [`saturating_mul_acc`][f-sma-1-8],
        [`wrapping_mul_acc`][f-wma-1-8], [`unwrapped_mul_acc`][f-uma-1-8],
        [`overflowing_mul_acc`][f-oma-1-8]
      * [`saturating_div_euclid_int`][f-sdei-1-8],
        [`saturating_rem_euclid_int`][f-srei-1-8]
      * [`unwrapped_rem`][f-ur-1-8], [`unwrapped_rem_euclid`][f-ure-1-8]
      * [`unwrapped_rem_int`][f-uri-1-8]
  * The following methods are now `const` functions:
      * [`checked_rem`][f-cr-1-8]
      * [`rem_euclid`][f-re-1-8], [`checked_rem_euclid`][f-cre-1-8]
      * [`checked_div_int`][f-cdi-1-8], [`wrapping_div_int`][f-wdi-1-8],
        [`unwrapped_div_int`][f-udi-1-8], [`overflowing_div_int`][f-odi-1-8]
  * The following methods were added to all fixed-point numbers:
      * [`const_not`][f-cn-1-8]
      * [`const_bitand`][f-cba-1-8], [`const_bitor`][f-cbo-1-8],
        [`const_bitxor`][f-cbx-1-8]
  * Many methods were marked with the `must_use` attribute.

[f-cba-1-8]: https://docs.rs/fixed/~1.8/fixed/struct.FixedI32.html#method.const_bitand
[f-cbo-1-8]: https://docs.rs/fixed/~1.8/fixed/struct.FixedI32.html#method.const_bitor
[f-cbx-1-8]: https://docs.rs/fixed/~1.8/fixed/struct.FixedI32.html#method.const_bitxor
[f-cdi-1-8]: https://docs.rs/fixed/~1.8/fixed/struct.FixedI32.html#method.checked_div_int
[f-cma-1-8]: https://docs.rs/fixed/~1.8/fixed/struct.FixedI32.html#method.checked_mul_acc
[f-cn-1-8]: https://docs.rs/fixed/~1.8/fixed/struct.FixedI32.html#method.const_not
[f-cr-1-8]: https://docs.rs/fixed/~1.8/fixed/struct.FixedI32.html#method.checked_rem
[f-cre-1-8]: https://docs.rs/fixed/~1.8/fixed/struct.FixedI32.html#method.checked_rem_euclid
[f-d-1-8]: https://docs.rs/fixed/~1.8/fixed/struct.FixedI32.html#associatedconstant.DELTA
[f-ma-1-8]: https://docs.rs/fixed/~1.8/fixed/struct.FixedI32.html#method.mul_acc
[f-o-1-8]: https://docs.rs/fixed/~1.8/fixed/struct.FixedI32.html#associatedconstant.ONE
[f-odi-1-8]: https://docs.rs/fixed/~1.8/fixed/struct.FixedI32.html#method.overflowing_div_int
[f-oma-1-8]: https://docs.rs/fixed/~1.8/fixed/struct.FixedI32.html#method.overflowing_mul_acc
[f-re-1-8]: https://docs.rs/fixed/~1.8/fixed/struct.FixedI32.html#method.rem_euclid
[f-sdei-1-8]: https://docs.rs/fixed/~1.8/fixed/struct.FixedI32.html#method.saturating_div_euclid_int
[f-sma-1-8]: https://docs.rs/fixed/~1.8/fixed/struct.FixedI32.html#method.saturating_mul_acc
[f-srei-1-8]: https://docs.rs/fixed/~1.8/fixed/struct.FixedI32.html#method.saturating_rem_euclid_int
[f-udi-1-8]: https://docs.rs/fixed/~1.8/fixed/struct.FixedI32.html#method.unwrapped_div_int
[f-uma-1-8]: https://docs.rs/fixed/~1.8/fixed/struct.FixedI32.html#method.unwrapped_mul_acc
[f-ur-1-8]: https://docs.rs/fixed/~1.8/fixed/struct.FixedI32.html#method.unwrapped_rem
[f-ure-1-8]: https://docs.rs/fixed/~1.8/fixed/struct.FixedI32.html#method.unwrapped_rem_euclid
[f-uri-1-8]: https://docs.rs/fixed/~1.8/fixed/struct.FixedI32.html#method.unwrapped_rem_int
[f-wdi-1-8]: https://docs.rs/fixed/~1.8/fixed/struct.FixedI32.html#method.wrapping_div_int
[f-wma-1-8]: https://docs.rs/fixed/~1.8/fixed/struct.FixedI32.html#method.wrapping_mul_acc
[f-z-1-8]: https://docs.rs/fixed/~1.8/fixed/struct.FixedI32.html#associatedconstant.ZERO
[tf-1-8]: https://docs.rs/fixed/~1.8/fixed/traits/trait.Fixed.html
[u-1-8]: https://docs.rs/fixed/~1.8/fixed/struct.Unwrapped.html
[w-1-8]: https://docs.rs/fixed/~1.8/fixed/struct.Wrapping.html

Version 1.7.0 (2021-03-25)
==========================

  * The crate now requires rustc version 1.50.0 or later.
  * The following methods are now `const` functions:
      * [`next_power_of_two`][f-npot-1-7],
        [`checked_next_power_of_two`][f-cnpot-1-7],
        [`wrapping_next_power_of_two`][f-wnpot-1-7]
  * The following constant and methods were added to all fixed-point numbers, to
    the [`Fixed`][tf-1-7] trait, and to the [`Wrapping`][w-1-7] and
    [`Unwrapped`][u-1-7] wrappers:
      * [`IS_SIGNED`][f-is-1-7]
      * [`from_be`][f-fb-1-7], [`from_le`][f-fl-1-7]
      * [`to_be`][f-tb-1-7], [`to_le`][f-tl-1-7]
      * [`swap_bytes`][f-sb-1-7]
      * [`reverse_bits`][f-rb-1-7]
      * [`mean`][f-m-1-7]
  * The following methods were added to the [`Wrapping`][w-1-7] and
    [`Unwrapped`][u-1-7] wrappers:
      * [`from_be_bytes`][f-fbb-1-7], [`from_le_bytes`][f-flb-1-7],
        [`from_ne_bytes`][f-fnb-1-7]
      * [`to_be_bytes`][f-tbb-1-7], [`to_le_bytes`][f-tlb-1-7],
        [`to_ne_bytes`][f-tnb-1-7]
  * The following methods were added to all unsigned fixed-point types, to the
    [`FixedUnsigned`][tfu-1-7] trait, and to the [`Wrapping`][w-1-7] and
    [`Unwrapped`][u-1-7] wrappers for unsigned numbers:
      * [`significant_bits`][f-signi-1-7]
      * [`highest_one`][f-ho-1-7]
  * The [`signed_bits`][f-signe-1-7] method was added to all signed fixed-point
    types, to the [`FixedSigned`][tfs-1-7] trait, and to the [`Wrapping`][w-1-7]
    and [`Unwrapped`][u-1-7] wrappers for signed numbers.
  * The following constants, which are available in other programming language
    standard libraries, were added to the [`consts`][c-1-7] module and as
    associated constants to fixed-point types:
      * [`SQRT_PI`][c-rp-1-7] (Go), [`FRAC_1_SQRT_PI`][c-1rp-1-7] (C++)
      * [`SQRT_3`][c-r3-1-7] (C++), [`FRAC_1_SQRT_3`][c-1r3-1-7] (C++)
      * [`SQRT_E`][c-re-1-7] (Go)
      * [`SQRT_PHI`][c-rf-1-7] (Go)
      * [`GAMMA`][c-g-1-7] (C++)
      * [`CATALAN`][c-c-1-7] (Julia)
  * [`Sum`] and [`Product`] are now supertraits of the [`Fixed`][tf-1-7] trait.
  * The [`F128Bits`][f128-1-7] type was added to support conversions and
    comparisons between fixed-point numbers and *binary128* floating-point
    numbers.
  * The features that previously required the [`az`][feat-dep-1-7] and
    [`f16`][feat-dep-1-7] optional features are now always provided. The
    [`az`][feat-dep-1-7] and [`f16`][feat-dep-1-7] optional features are now
    deprecated and have no effect.
  * For the experimental feature [`num-traits`][feat-exp-1-7], the following
    traits were implemented for all fixed-point numbers:
      * [`OverflowingAdd`][nt-0-2-oa], [`OverflowingSub`][nt-0-2-os],
        [`OverflowingMul`][nt-0-2-om]

[c-1-7]: https://docs.rs/fixed/~1.7/fixed/consts/index.html
[c-1r3-1-7]: https://docs.rs/fixed/~1.7/fixed/consts/constant.FRAC_1_SQRT_3.html
[c-1rp-1-7]: https://docs.rs/fixed/~1.7/fixed/consts/constant.FRAC_1_SQRT_PI.html
[c-c-1-7]: https://docs.rs/fixed/~1.7/fixed/consts/constant.CATALAN.html
[c-g-1-7]: https://docs.rs/fixed/~1.7/fixed/consts/constant.GAMMA.html
[c-r3-1-7]: https://docs.rs/fixed/~1.7/fixed/consts/constant.SQRT_3.html
[c-re-1-7]: https://docs.rs/fixed/~1.7/fixed/consts/constant.SQRT_E.html
[c-rf-1-7]: https://docs.rs/fixed/~1.7/fixed/consts/constant.SQRT_PHI.html
[c-rp-1-7]: https://docs.rs/fixed/~1.7/fixed/consts/constant.SQRT_PI.html
[f-cnpot-1-7]: https://docs.rs/fixed/~1.7/fixed/struct.FixedU32.html#method.checked_next_power_of_two
[f-fb-1-7]: https://docs.rs/fixed/~1.7/fixed/struct.FixedI32.html#method.from_be
[f-fbb-1-7]: https://docs.rs/fixed/~1.7/fixed/struct.FixedI32.html#method.from_be_bytes
[f-fl-1-7]: https://docs.rs/fixed/~1.7/fixed/struct.FixedI32.html#method.from_le
[f-flb-1-7]: https://docs.rs/fixed/~1.7/fixed/struct.FixedI32.html#method.from_le_bytes
[f-fnb-1-7]: https://docs.rs/fixed/~1.7/fixed/struct.FixedI32.html#method.from_ne_bytes
[f-ho-1-7]: https://docs.rs/fixed/~1.7/fixed/struct.FixedU32.html#method.highest_one
[f-is-1-7]: https://docs.rs/fixed/~1.7/fixed/struct.FixedI32.html#associatedconstant.IS_SIGNED
[f-m-1-7]: https://docs.rs/fixed/~1.7/fixed/struct.FixedI32.html#method.mean
[f-npot-1-7]: https://docs.rs/fixed/~1.7/fixed/struct.FixedU32.html#method.next_power_of_two
[f-rb-1-7]: https://docs.rs/fixed/~1.7/fixed/struct.FixedI32.html#method.reverse_bits
[f-sb-1-7]: https://docs.rs/fixed/~1.7/fixed/struct.FixedI32.html#method.swap_bytes
[f-signe-1-7]: https://docs.rs/fixed/~1.7/fixed/struct.FixedI32.html#method.signed_bits
[f-signi-1-7]: https://docs.rs/fixed/~1.7/fixed/struct.FixedU32.html#method.significant_bits
[f-tb-1-7]: https://docs.rs/fixed/~1.7/fixed/struct.FixedI32.html#method.to_be
[f-tbb-1-7]: https://docs.rs/fixed/~1.7/fixed/struct.FixedI32.html#method.to_be_bytes
[f-tl-1-7]: https://docs.rs/fixed/~1.7/fixed/struct.FixedI32.html#method.to_le
[f-tlb-1-7]: https://docs.rs/fixed/~1.7/fixed/struct.FixedI32.html#method.to_le_bytes
[f-tnb-1-7]: https://docs.rs/fixed/~1.7/fixed/struct.FixedI32.html#method.to_ne_bytes
[f-wnpot-1-7]: https://docs.rs/fixed/~1.7/fixed/struct.FixedU32.html#method.wrapping_next_power_of_two
[f128-1-7]: https://docs.rs/fixed/~1.7/fixed/struct.F128Bits.html
[feat-dep-1-7]: https://docs.rs/fixed/~1.7/fixed/index.html#deprecated-optional-features
[feat-exp-1-7]: https://docs.rs/fixed/~1.7/fixed/index.html#experimental-optional-features
[nt-0-2-oa]: https://docs.rs/num-traits/^0.2/num_traits/ops/overflowing/trait.OverflowingAdd.html
[nt-0-2-om]: https://docs.rs/num-traits/^0.2/num_traits/ops/overflowing/trait.OverflowingMul.html
[nt-0-2-os]: https://docs.rs/num-traits/^0.2/num_traits/ops/overflowing/trait.OverflowingSub.html
[tf-1-7]: https://docs.rs/fixed/~1.7/fixed/traits/trait.Fixed.html
[tfs-1-7]: https://docs.rs/fixed/~1.7/fixed/traits/trait.FixedSigned.html
[tfu-1-7]: https://docs.rs/fixed/~1.7/fixed/traits/trait.FixedUnsigned.html
[u-1-7]: https://docs.rs/fixed/~1.7/fixed/struct.Unwrapped.html
[w-1-7]: https://docs.rs/fixed/~1.7/fixed/struct.Wrapping.html

Version 1.6.0 (2021-02-05)
==========================

  * The crate now requires rustc version 1.47.0 or later.
  * The optional [*az* crate] dependency was updated to [version 1.1][az-1-1].
  * The [`unsigned_abs`][f-ua-1-6] method was added to all signed fixed-point
    types and to the [`FixedSigned`][tfs-1-6] trait.
  * The following methods are now `const` functions:
      * [`checked_neg`][f-cn-1-6], [`checked_add`][f-cad-1-6],
        [`checked_sub`][f-cs-1-6], [`checked_mul_int`][f-cmi-1-6],
        [`checked_shl`][f-cshl-1-6], [`checked_shr`][f-cshr-1-6],
        [`checked_abs`][f-cab-1-6]
  * The [`unwrapped_to_fixed`][f-utf-1-6] method was added to the
    [`ToFixed`][f-tf-1-6] trait.
  * The [`unwrapped_from_fixed`][f-uff-1-6] method was added to the
    [`FromFixed`][f-ff-1-6] trait.

[az-1-1]: https://docs.rs/az/~1.1/az/index.html
[f-cab-1-6]: https://docs.rs/fixed/~1.6/fixed/struct.FixedI32.html#method.checked_abs
[f-cad-1-6]: https://docs.rs/fixed/~1.6/fixed/struct.FixedI32.html#method.checked_add
[f-cmi-1-6]: https://docs.rs/fixed/~1.6/fixed/struct.FixedI32.html#method.checked_mul_int
[f-cn-1-6]: https://docs.rs/fixed/~1.6/fixed/struct.FixedI32.html#method.checked_neg
[f-cs-1-6]: https://docs.rs/fixed/~1.6/fixed/struct.FixedI32.html#method.checked_sub
[f-cshl-1-6]: https://docs.rs/fixed/~1.6/fixed/struct.FixedI32.html#method.checked_shl
[f-cshr-1-6]: https://docs.rs/fixed/~1.6/fixed/struct.FixedI32.html#method.checked_shr
[f-ff-1-6]: https://docs.rs/fixed/~1.6/fixed/traits/trait.FromFixed.html
[f-tf-1-6]: https://docs.rs/fixed/~1.6/fixed/traits/trait.ToFixed.html
[f-ua-1-6]: https://docs.rs/fixed/~1.6/fixed/struct.FixedI32.html#method.unsigned_abs
[f-uff-1-6]: https://docs.rs/fixed/~1.6/fixed/traits/trait.FromFixed.html#method.unwrapped_from_fixed
[f-utf-1-6]: https://docs.rs/fixed/~1.6/fixed/traits/trait.ToFixed.html#method.unwrapped_to_fixed
[tfs-1-6]: https://docs.rs/fixed/~1.6/fixed/traits/trait.FixedSigned.html

Version 1.5.0 (2020-11-05)
==========================

  * The [`wide_mul`][f-wm-1-5] method was added to all fixed-point numbers up to
    64 bits wide ([issue 25]).
  * Unwrapped methods for arithmetic together with the [`Unwrapped`][unw-1-5]
    wrapper were added. Unwrapped methods panic on overflow, even when debug
    assertions are disabled, similar to how wrapping methods will wrap around
    even when debug assertions are enabled. (This was previously an experimental
    feature `unwrapped`.)
  * The [`serde-str`][feat-1-5] feature was added. (This was previously an
    experimental feature.)
  * For the experimental feature [`num-traits`][feat-exp-1-5], some missing
    supertraits were added to [`FixedOptionalFeatures`][tfof-1-5].
  * Bug fix: multiplication of [`FixedI128`][i128-1-5] was panicking when
    multiplying some large negative numbers ([issue 26]).

[i128-1-5]: https://docs.rs/fixed/~1.5/fixed/struct.FixedI128.html
[f-wm-1-5]: https://docs.rs/fixed/~1.5/fixed/struct.FixedI32.html#method.wide_mul
[feat-1-5]: https://docs.rs/fixed/~1.5/fixed/index.html#optional-features
[feat-exp-1-5]: https://docs.rs/fixed/~1.5/fixed/index.html#experimental-optional-features
[issue 25]: https://gitlab.com/tspiteri/fixed/-/issues/25
[issue 26]: https://gitlab.com/tspiteri/fixed/-/issues/26
[tfof-1-5]: https://docs.rs/fixed/~1.5/fixed/traits/trait.FixedOptionalFeatures.html
[unw-1-5]: https://docs.rs/fixed/~1.5/fixed/struct.Unwrapped.html

Version 1.4.0 (2020-10-22)
==========================

  * The following methods were added to all fixed-point types, to the
    [`Fixed`][tf-1-4] trait, and to the [`Wrapping`][w-1-4] wrapper:
      * [`recip`][f-rec-1-4], [`checked_recip`][f-crec-1-4],
        [`saturating_recip`][f-srec-1-4], [`wrapping_recip`][f-wrec-1-4],
        [`overflowing_recip`][f-orec-1-4]
  * For the experimental feature [`num-traits`][feat-exp-1-4], the following
    traits were implemented where applicable ([issue 23]):
      * [`Num`][nt-0-2-num]
      * [`Signed`][nt-0-2-signed], [`Unsigned`][nt-0-2-unsigned]
      * [`Inv`][nt-0-2-inv]
      * [`MulAdd`][nt-0-2-ma], [`MulAddAssign`][nt-0-2-maa]
      * [`FloatConst`][nt-0-2-fc]
      * [`ToPrimitive`][nt-0-2-tp], [`FromPrimitive`][nt-0-2-fp]
  * For the experimental feature [`serde-str`][feat-exp-1-4], serialization in
    human-readable formats was made more convenient to write manually ([issue
    24]). This makes it incompatible with the version in 1.3.0.

[f-crec-1-4]: https://docs.rs/fixed/~1.4/fixed/struct.FixedI32.html#method.checked_recip
[f-orec-1-4]: https://docs.rs/fixed/~1.4/fixed/struct.FixedI32.html#method.overflowing_recip
[f-rec-1-4]: https://docs.rs/fixed/~1.4/fixed/struct.FixedI32.html#method.recip
[f-srec-1-4]: https://docs.rs/fixed/~1.4/fixed/struct.FixedI32.html#method.saturating_recip
[f-wrec-1-4]: https://docs.rs/fixed/~1.4/fixed/struct.FixedI32.html#method.wrapping_recip
[feat-exp-1-4]: https://docs.rs/fixed/~1.4/fixed/index.html#experimental-optional-features
[issue 23]: https://gitlab.com/tspiteri/fixed/-/issues/23
[issue 24]: https://gitlab.com/tspiteri/fixed/-/issues/24
[nt-0-2-fc]: https://docs.rs/num-traits/^0.2/num_traits/float/trait.FloatConst.html
[nt-0-2-fp]: https://docs.rs/num-traits/^0.2/num_traits/cast/trait.FromPrimitive.html
[nt-0-2-inv]: https://docs.rs/num-traits/^0.2/num_traits/ops/inv/trait.Inv.html
[nt-0-2-ma]: https://docs.rs/num-traits/^0.2/num_traits/ops/mul_add/trait.MulAdd.html
[nt-0-2-maa]: https://docs.rs/num-traits/^0.2/num_traits/ops/mul_add/trait.MulAddAssign.html
[nt-0-2-num]: https://docs.rs/num-traits/^0.2/num_traits/trait.Num.html
[nt-0-2-signed]: https://docs.rs/num-traits/^0.2/num_traits/sign/trait.Signed.html
[nt-0-2-tp]: https://docs.rs/num-traits/^0.2/num_traits/cast/trait.ToPrimitive.html
[nt-0-2-unsigned]: https://docs.rs/num-traits/^0.2/num_traits/sign/trait.Unsigned.html
[tf-1-4]: https://docs.rs/fixed/~1.4/fixed/traits/trait.Fixed.html
[w-1-4]: https://docs.rs/fixed/~1.4/fixed/struct.Wrapping.html

Version 1.3.0 (2020-10-15)
==========================

  * The [`MulAssign`] implementation on fixed-point numbers now accepts an rhs
    fixed-point number with a different number of fractional bits from `self`.
  * The following methods were added to all fixed-point types, to the
    [`Fixed`][tf-1-3] trait, and to the [`Wrapping`][w-1-3] wrapper:
      * [`mul_add`][f-ma-1-3], [`checked_mul_add`][f-cma-1-3],
        [`saturating_mul_add`][f-sma-1-3], [`wrapping_mul_add`][f-wma-1-3],
        [`overflowing_mul_add`][f-oma-1-3]
  * The new experimental feature [`unwrapped`][feat-exp-1-3] was added,
    providing arithmetic methods that panic on overflow even when debug
    assertions are disabled.
  * The new experimental feature [`serde-str`][feat-exp-1-3] was added, which
    makes serialization use the number’s value in human-readable formats.

[f-cma-1-3]: https://docs.rs/fixed/~1.3/fixed/struct.FixedI32.html#method.checked_mul_add
[f-ma-1-3]: https://docs.rs/fixed/~1.3/fixed/struct.FixedI32.html#method.mul_add
[f-oma-1-3]: https://docs.rs/fixed/~1.3/fixed/struct.FixedI32.html#method.overflowing_mul_add
[f-sma-1-3]: https://docs.rs/fixed/~1.3/fixed/struct.FixedI32.html#method.saturating_mul_add
[f-wma-1-3]: https://docs.rs/fixed/~1.3/fixed/struct.FixedI32.html#method.wrapping_mul_add
[feat-exp-1-3]: https://docs.rs/fixed/~1.3/fixed/index.html#experimental-optional-features
[tf-1-3]: https://docs.rs/fixed/~1.3/fixed/traits/trait.Fixed.html
[w-1-3]: https://docs.rs/fixed/~1.3/fixed/struct.Wrapping.html

Version 1.2.0 (2020-09-02)
==========================

  * The [`const_fixed_from_int!`][cffi-1-2] macro was added to make it easy to
    define constant fixed-point numbers using integer expressions ([issue 20]).

[cffi-1-2]: https://docs.rs/fixed/~1.2/fixed/macro.const_fixed_from_int.html
[issue 20]: https://gitlab.com/tspiteri/fixed/-/issues/20

Version 1.1.0 (2020-07-21)
==========================

  * The new experimental feature [`num-traits`][feat-nt-1-1] was added to
    implement some traits, and to also add the relevant traits as supertraits to
    the [`FixedOptionalFeatures`][fof-1-1] trait ([issue 18]).

[issue 18]: https://gitlab.com/tspiteri/fixed/-/issues/18
[feat-nt-1-1]: https://docs.rs/fixed/~1.1/fixed/index.html#experimental-optional-features
[fof-1-1]: https://docs.rs/fixed/~1.1/fixed/traits/trait.FixedOptionalFeatures.html

Version 1.0.0 (2020-06-04)
==========================

  * The crate now requires rustc version 1.44.0 or later.
  * The following methods are now `const` functions:
      * [`from_be_bytes`][f-fbb-1-0], [`from_le_bytes`][f-flb-1-0],
        [`from_ne_bytes`][f-fnb-1-0]
      * [`to_be_bytes`][f-tbb-1-0], [`to_le_bytes`][f-tlb-1-0],
        [`to_ne_bytes`][f-tnb-1-0]
  * All deprecated items were removed.

[f-fbb-1-0]: https://docs.rs/fixed/~1.0/fixed/struct.FixedI32.html#method.from_be_bytes
[f-flb-1-0]: https://docs.rs/fixed/~1.0/fixed/struct.FixedI32.html#method.from_le_bytes
[f-fnb-1-0]: https://docs.rs/fixed/~1.0/fixed/struct.FixedI32.html#method.from_ne_bytes
[f-tbb-1-0]: https://docs.rs/fixed/~1.0/fixed/struct.FixedI32.html#method.to_be_bytes
[f-tlb-1-0]: https://docs.rs/fixed/~1.0/fixed/struct.FixedI32.html#method.to_le_bytes
[f-tnb-1-0]: https://docs.rs/fixed/~1.0/fixed/struct.FixedI32.html#method.to_ne_bytes

Version 0.5.7 (2020-05-11)
==========================

  * The `LosslessTryFrom` and `LosslessTryInto` traits were added.
  * The following methods were added to all fixed-point types, to the `Fixed`
    trait, and to the `Wrapping` wrapper:
      * `leading_ones`, `trailing_ones`
  * The following method was added to unsigned fixed-point types and to the
    `FixedUnsigned` trait:
      * `wrapping_next_power_of_two`
  * The `PHI` and `FRAC_1_PHI` constants were added to the `consts` module and
    as associated constants to fixed-point types.

Version 0.5.6 (2020-05-01)
==========================

  * The following methods were added to signed fixed-point types and to the
    `FixedSigned` trait:
      * `checked_signum`, `saturating_signum`, `wrapping_signum`,
        `overflowing_signum`
  * The `LossyFrom` and `LossyInto` traits were added to the prelude.
  * Casts deprecated in version 0.3.1 of the *az* crate were marked as
    deprecated.

Version 0.5.5 (2020-04-16)
==========================

  * Bug fix: an incorrect result could be given when comparing a signed
    fixed-point number of type `FixedI` to a number that would overflow by
    exactly one bit when converting to `FixedI`.
  * The following associated constants were added to all fixed-point types, to
    the `Fixed` trait, and to the `Wrapping` wrapper:
      * `MIN`, `MAX`
  * The following associated constants were added to the `Fixed` trait and to
    the `Wrapping` wrapper:
      * `INT_NBITS`, `FRAC_NBITS`
  * The following methods were added to all fixed-point types and to the `Fixed`
    trait:
      * `int_log2`, `int_log10`
      * `checked_int_log2`, `checked_int_log10`
  * The following methods were added to the `Wrapping` wrapper:
      *  `int_log2`, `int_log10`
  * The constants in the `consts` module were also added as associated constants
    to fixed-point types that can represent them.
  * The following methods were deprecated:
      * `min_value`, `max_value`
      * `int_nbits`, `frac_nbits`

Version 0.5.4 (2020-02-21)
==========================

  * Bug fix: `rem_euclid_int` and its checked versions were handling overflow
    incorrectly.

Version 0.5.3 (2020-02-13)
==========================

  * Bug fix: `round_to_zero` was returning incorrect results for negative whole
    number operands.
  * Bug fix: all remainder operations with a fixed-point LHS and an integer RHS
    were giving an incorrect answer
    (https://gitlab.com/tspiteri/fixed/issues/13).
  * Bug fix: Euclidean division operations by integers were giving an incorrect
    answer.
  * `Rem` and `RemAssign` were implemented for fixed-point numbers.
  * The following methods were added to all fixed-point types and to the `Fixed`
    trait:
      * `checked_rem`
      * `div_euclid`, `rem_euclid`
      * `checked_div_euclid`, `checked_rem_euclid`
      * `saturating_div_euclid`
      * `wrapping_div_euclid`
      * `overflowing_div_euclid`
  * The following methods were added to the `Wrapping` wrapper:
      * `div_euclid`, `rem_euclid`
      * `div_euclid_int`, `rem_euclid_int`
  * The following methods were deprecated:
      * `wrapping_rem_int`, `overflowing_rem_int`

Version 0.5.2 (2020-02-02)
==========================

  * `Wrapping` now supports serialization. (Thanks: Shane Pearman)

Version 0.5.1 (2019-12-22)
==========================

  * `ParseFixedError` implements `Error` when the new `std` feature is enabled.

Version 0.5.0 (2019-12-06)
==========================

  * The crate now requires rustc version 1.39.0 or later.
  * The following methods were added to all fixed-point types and to the `Fixed`
    trait:
      * `from_be_bytes`, `from_le_bytes`, `from_ne_bytes`
      * `to_be_bytes`, `to_le_bytes`, `to_ne_bytes`
      * `div_euclid_int`, `rem_euclid_int`
      * `checked_div_euclid_int`, `checked_rem_euclid_int`
      * `wrapping_div_euclid_int`, `wrapping_rem_euclid_int`
      * `overflowing_div_euclid_int`, `overflowing_rem_euclid_int`
  * Casts deprecated in version 0.3.1 of the *az* crate were marked as
    deprecated.

Incompatible changes
--------------------

  * Deprecated methods and modules were removed.

Version 0.4.6 (2019-10-16)
==========================

  * Conversions to/from `bf16` are now provided when the `f16` option is
    enabled.
  * The following methods are now `const` functions: `saturating_neg`,
    `saturating_add`, `saturating_sub`, `saturating_mul_int`, `saturating_abs`
  * Support for casts using the *az* crate was added.

Version 0.4.5 (2019-08-30)
==========================

  * Bug fix: display of many decimal numbers was panicking in debug mode or
    including a leading zero in release mode.
  * Many methods were added to `Wrapping` for convenience, even if they do not
    involve wrapping.

Version 0.4.4 (2019-08-24)
==========================

  * Bug fix: rounding could produce bad output for `Binary`, `Octal`, `LowerHex`
    and `UpperHex`.
  * The following methods are now `const` functions: `is_power_of_two`, `abs`,
    `wrapping_abs`, `overflowing_abs`
  * The method `round_to_zero` was added.
  * The method `round_ties_to_even` and its checked versions were added.

Version 0.4.3 (2019-08-20)
==========================

  * The crate now requires rustc version 1.34.0 or later.
  * The precision argument is no longer ignored when formatting fixed-point
    numbers; precision should now be handled the same as for primitive
    floating-point numbers in the standard library.
  * Parsing strings now rounds to the nearest with ties rounded to even.
  * Checked versions of string parsing methods are now available as inherent
    methods to all fixed-point numbers, and as methods in the `Fixed` trait.
  * `Wrapping` now has methods for parsing with wrapping, including an
    implementation of `FromStr`.
  * The following methods are now `const` functions:
      * `min_value`, `max_value`, `from_bits`, `to_bits`
      * `count_ones`, `count_zeros`, `leading_zeros`, `trailing_zeros`
        `rotate_left`, `rotate_right`
      * `wrapping_neg`, `wrapping_add`, `wrapping_sub`, `wrapping_mul_int`,
        `wrapping_shl`, `wrapping_shr`
      * `overflowing_neg`, `overflowing_add`, `overflowing_sub`,
        `overflowing_mul_int`, `overflowing_shl`, `overflowing_shr`
      * `is_positive`, `is_negative`
  * The associated constants `INT_NBITS` and `FRAC_NBITS` were added.
  * The reexports in the `frac` module and the `LeEqU*` traits were moved into
    the new `types::extra` module.

Version 0.4.2 (2019-08-16)
==========================

  * The new methods `from_num` and `to_num` together with their checked versions
    were added to all fixed-point numbers.
  * The methods `from_fixed`, `to_fixed`, `from_int`, `to_int`, `from_float`,
    and `to_float`, and their checked versions, were deprecated.
  * The new method `from_num` was added to the `Wrapping` wrapper.
  * Bug fix: parsing of decimal fractions was fixed to give correctly rounded
    results for long decimal fraction strings, for example with four fractional
    bits, 0.96874999… (just below 31⁄32) and 0.96875 (31⁄32) are now parsed
    correctly as 0.9375 (15⁄16) and 1.0.

Version 0.4.1 (2019-08-12)
==========================

  * All fixed-point types now implement `FromStr`.
  * The methods `from_str_binary`, `from_str_octal` and `from_str_hex` were
    added.

Version 0.4.0 (2019-08-08)
==========================

  * The crate now requires rustc version 1.31.0 or later.
  * The `traits` module was added, with its traits `Fixed`, `FixedSigned`,
    `FixedUnsigned`, `FromFixed`, `ToFixed`, `LossyFrom` and `LossyInto`.
  * The `saturating_neg` method was added to all fixed-point numbers, and the
    `saturating_abs` method was added to signed fixed-point numbers.
  * The `consts` module was added.
  * The `signum` method now wraps instead of panics in release mode.

Incompatible changes
--------------------

  * The sealed traits `Int` and `Float` now have no provided methods; the
    methods in the old implementation are now provided by `FromFixed` and
    `ToFixed`.
  * Deprecated methods were removed.

Contributors
------------

  * @jean-airoldie
  * @tspiteri

Version 0.3.3 (2019-06-27)
==========================

  * Conversions to/from `isize` and `usize` were added.

Version 0.3.2 (2019-02-27)
==========================

  * The `Wrapping` wrapper was added.

Version 0.3.1 (2019-02-07)
==========================

  * Reimplement `From<bool>` for all fixed-point types which can represent the
    integer 1. This was inadvertently removed in 0.3.0.

Version 0.3.0 (2019-02-03)
==========================

  * Incompatible change: the return type of `to_int` is now generic.
  * Incompatible change: the `Int` trait implementation for `bool` was removed.
  * The new method `to_fixed` was added.
  * The new methods `checked_to_fixed`, `checked_to_int`, `saturating_to_fixed`,
    `saturating_to_int`, `wrapping_to_fixed`, `wrapping_to_int`,
    `overflowing_to_fixed` and `overflowing_to_int` were added.
  * The methods `from_fixed`, `to_fixed`, `checked_from_fixed`,
    `checked_to_fixed`, `saturating_from_fixed`, `saturating_to_fixed`,
    `wrapping_from_fixed`, `wrapping_to_fixed`, `overflowing_from_fixed` and
    `overflowing_to_fixed` were added to the `Int` trait.
  * The methods `from_fixed`, `to_fixed`, `checked_to_fixed`,
    `saturating_to_fixed`, `wrapping_to_fixed` and `overflowing_to_fixed` were
    added to the `Float` trait.
  * `PartialEq` and `PartialCmp` are now implemented for all combinations of
    fixed-point numbers and primitive integers.
  * The methods `int_bits` and `frac_bits` were deprecated and replaced by the
    methods `int_nbits` and `frac_nbits`.

Version 0.2.1 (2019-01-29)
==========================

  * Bug fix: the `from_fixed` and `from_int` methods (and their checked
    counterparts) could return wrong values for negative values.
  * Bug fix: display was using one fractional digit less than required, thus
    yielding the same output for diffent fixed-point numbers.

Version 0.2.0 (2019-01-29)
==========================

  * Incompatible change: The method `from_int` was change to accept a generic
    parameter.
  * The new methods `from_fixed`, `checked_from_fixed`, `saturating_from_fixed`,
    `wrapping_from_fixed` and `overflowing_from_fixed` were added.
  * The new methods `checked_from_int`, `saturating_from_int`,
    `wrapping_from_int` and `overflowing_from_int` were added.
  * The new methods `from_float`, `checked_from_float`, `saturating_from_float`,
    `wrapping_from_float` and `overflowing_from_float` were added.
  * The new method `to_float` was added.
  * The methods `from_f16`, `from_f32`, `from_f64`, `to_f16`, `to_f32` and
    `to_f64` were deprecated.
  * The `to_int` method was fixed to truncate fractional bits as documented for
    negative values.
  * The new methods `ceil`, `floor`, `round`, `checked_ceil`, `checked_floor`,
    `checked_round`, `saturating_ceil`, `saturating_floor`, `saturating_round`,
    `wrapping_ceil`, `wrapping_floor`, `wrapping_round`, `overflowing_ceil`,
    `overflowing_floor` and `overflowing_round` were added.
  * The methods `to_int_ceil`, `to_int_floor` and `to_int_round` were
    deprecated.

Version 0.1.6 (2019-01-27)
==========================

  * Optional serde support was added.

Version 0.1.5 (2019-01-26)
==========================

  * Lossless infallible conversions between fixed-point numbers and numeric
    primitives are now supported using `From` and `Into`.
  * A new module `types` is available with aliases for all supported fixed-point
    numbers.

Version 0.1.4 (2018-11-29)
==========================

  * Division is now implemented for `FixedI128` and `FixedU128`.

Version 0.1.3 (2018-08-23)
==========================

  * The `f16` feature was added, and new methods `from_f16` and `to_f16` were
    added.

Version 0.1.2 (2018-08-15)
==========================

  * The crate can now be used without the standard library `std`.
  * New methods `from_f32` and `from_f64` were added.
  * New methods `is_positive` and `is_negative` were added to signed fixed-point
    numbers.

Version 0.1.1 (2018-08-11)
==========================

  * Comparisons are now supported between all fixed-point numbers with the same
    underlying integer type.
  * New static methods `int_bits` and `frac_bits` were added.
  * New methods `from_int`, `to_int`, `to_int_ceil`, `to_int_floor` and
    `to_int_round` were added.
  * New methods `int` and `frac` were added.
  * Support for multiplication and division by integers was added.

Version 0.1.0 (2018-08-10)
==========================

  * `Unsigned` constants provided by the *typenum* crate are now used for the
    number of fractional bits.
  * Many methods and trait implementations available for primitive integers are
    now also supported by the fixed-point numbers.

[*arbitrary* crate]: https://crates.io/crates/arbitrary
[*az* crate]: https://crates.io/crates/az
[*borsh* crate]: https://crates.io/crates/borsh
[*bytemuck* crate]: https://crates.io/crates/bytemuck
[*typenum* crate]: https://crates.io/crates/typenum
[`Debug`]: https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html
[`DivAssign`]: https://doc.rust-lang.org/nightly/core/ops/trait.DivAssign.html
[`Div`]: https://doc.rust-lang.org/nightly/core/ops/trait.Div.html
[`MulAssign`]: https://doc.rust-lang.org/nightly/core/ops/trait.MulAssign.html
[`NonZeroI32`]: https://doc.rust-lang.org/nightly/core/num/struct.NonZeroI32.html
[`NonZeroU32`]: https://doc.rust-lang.org/nightly/core/num/struct.NonZeroU32.html
[`Product`]: https://doc.rust-lang.org/nightly/core/iter/trait.Product.html
[`RemAssign`]: https://doc.rust-lang.org/nightly/core/ops/trait.RemAssign.html
[`Rem`]: https://doc.rust-lang.org/nightly/core/ops/trait.Rem.html
[`Sum`]: https://doc.rust-lang.org/nightly/core/iter/trait.Sum.html
