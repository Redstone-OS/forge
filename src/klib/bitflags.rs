/// Macro `bitflags!` minimalista para substituir a create externa `bitflags`.
///
/// Permite definir estruturas que representam conjuntos de flags (bits).
/// Implementa operações básicas de bitwise e verificação de flags.
///
/// # Exemplo
///
/// ```rust
/// bitflags! {
///     pub struct Flags: u32 {
///         const A = 0b00000001;
///         const B = 0b00000010;
///         const C = 0b00000100;
///     }
/// }
/// ```
#[macro_export]
macro_rules! bitflags {
    (
        $(#[$outer:meta])*
        pub struct $BitFlags:ident: $T:ty {
            $(
                $(#[$inner:ident $($args:tt)*])*
                const $Flag:ident = $value:expr;
            )+
        }
    ) => {
        $(#[$outer])*
        #[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
        #[repr(transparent)]
        pub struct $BitFlags($T);

        impl $BitFlags {
            $(
                $(#[$inner $($args)*])*
                pub const $Flag: $BitFlags = $BitFlags($value);
            )+

            /// Cria um conjunto de flags vazio.
            #[inline]
            pub const fn empty() -> Self {
                Self(0)
            }

            /// Cria um conjunto com todas as flags (união de todas as constantes definidas).
            #[inline]
            pub const fn all() -> Self {
                Self(0 $ ( | $value )+)
            }

            /// Retorna o valor bruto subjacente.
            #[inline]
            pub const fn bits(&self) -> $T {
                self.0
            }

            /// Cria flags a partir de bits brutos, sem verificação.
            #[inline]
            pub const fn from_bits_truncate(bits: $T) -> Self {
                Self(bits)
            }

            /// Cria flags a partir de bits se for válido (apenas contém flags definidas).
            /// Nesta implementação simplificada, aceitamos quaisquer bits para compatibilidade.
            #[inline]
            pub const fn from_bits(bits: $T) -> Option<Self> {
                Some(Self(bits))
            }

            /// Verifica se o conjunto contém todas as flags especificadas.
            #[inline]
            pub const fn contains(&self, other: Self) -> bool {
                (self.0 & other.0) == other.0
            }

            /// Insere as flags especificadas.
            #[inline]
            pub fn insert(&mut self, other: Self) {
                self.0 |= other.0;
            }

            /// Remove as flags especificadas.
            #[inline]
            pub fn remove(&mut self, other: Self) {
                self.0 &= !other.0;
            }

            /// Alterna as flags especificadas.
            #[inline]
            pub fn toggle(&mut self, other: Self) {
                self.0 ^= other.0;
            }

            /// Retorna a interseção entre dois conjuntos de flags.
            #[inline]
            pub const fn intersection(self, other: Self) -> Self {
                Self(self.0 & other.0)
            }

            /// Retorna a união entre dois conjuntos de flags.
            #[inline]
            pub const fn union(self, other: Self) -> Self {
                Self(self.0 | other.0)
            }

            /// Retorna a diferença (self - other).
            #[inline]
            pub const fn difference(self, other: Self) -> Self {
                Self(self.0 & !other.0)
            }

            /// Retorna o complemento das flags.
            #[inline]
            pub const fn complement(self) -> Self {
                Self(!self.0)
            }
        }

        impl core::ops::BitOr for $BitFlags {
            type Output = Self;
            #[inline]
            fn bitor(self, rhs: Self) -> Self {
                self.union(rhs)
            }
        }

        impl core::ops::BitOrAssign for $BitFlags {
            #[inline]
            fn bitor_assign(&mut self, rhs: Self) {
                self.insert(rhs);
            }
        }

        impl core::ops::BitAnd for $BitFlags {
            type Output = Self;
            #[inline]
            fn bitand(self, rhs: Self) -> Self {
                self.intersection(rhs)
            }
        }

        impl core::ops::BitAndAssign for $BitFlags {
            #[inline]
            fn bitand_assign(&mut self, rhs: Self) {
                self.0 &= rhs.0;
            }
        }

        impl core::ops::BitXor for $BitFlags {
            type Output = Self;
            #[inline]
            fn bitxor(self, rhs: Self) -> Self {
                Self(self.0 ^ rhs.0)
            }
        }

        impl core::ops::BitXorAssign for $BitFlags {
            #[inline]
            fn bitxor_assign(&mut self, rhs: Self) {
                self.toggle(rhs);
            }
        }

        impl core::ops::Not for $BitFlags {
            type Output = Self;
            #[inline]
            fn not(self) -> Self {
                self.complement()
            }
        }
    };
}
