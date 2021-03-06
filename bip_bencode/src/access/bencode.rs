use access::dict::BDictAccess;
use access::list::BListAccess;

/// Abstract representation of a `BencodeRef` object.
pub enum BencodeRefKind<'b, 'a: 'b, T: 'b> {
    /// Bencode Integer.
    Int(i64),
    /// Bencode Bytes.
    Bytes(&'a [u8]),
    /// Bencode List.
    List(&'b BListAccess<T>),
    /// Bencode Dictionary.
    Dict(&'b BDictAccess<'a, T>),
}

/// Trait for read access to some bencode type.
pub trait BRefAccess<'a>: Sized {
    type BType: BRefAccess<'a>;

    /// Access the bencode as a `BencodeRefKind`.
    fn kind<'b>(&'b self) -> BencodeRefKind<'b, 'a, Self::BType>;

    /// Attempt to access the bencode as a `str`.
    fn str(&self) -> Option<&'a str>;

    /// Attempt to access the bencode as an `i64`.
    fn int(&self) -> Option<i64>;

    /// Attempt to access the bencode as an `[u8]`.
    fn bytes(&self) -> Option<&'a [u8]>;

    /// Attempt to access the bencode as an `BListAccess`.
    fn list(&self) -> Option<&BListAccess<Self::BType>>;

    /// Attempt to access the bencode as an `BDictAccess`.
    fn dict(&self) -> Option<&BDictAccess<'a, Self::BType>>;
}

impl<'a: 'b, 'b, T> BRefAccess<'a> for &'b T where T: BRefAccess<'a> {
    type BType = T::BType;

    fn kind<'c>(&'c self) -> BencodeRefKind<'c, 'a, Self::BType> {
        (*self).kind()
    }

    fn str(&self) -> Option<&'a str> {
        (*self).str()
    }

    fn int(&self) -> Option<i64> {
        (*self).int()
    }

    fn bytes(&self) -> Option<&'a [u8]> {
        (*self).bytes()
    }

    fn list(&self) -> Option<&BListAccess<Self::BType>> {
        (*self).list()
    }

    fn dict(&self) -> Option<&BDictAccess<'a, Self::BType>> {
        (*self).dict()
    }
}

/// Abstract representation of a `BencodeMut` object.
pub enum BencodeMutKind<'b, 'a: 'b, T: 'b> {
    /// Bencode Integer.
    Int(i64),
    /// Bencode Bytes.
    Bytes(&'a [u8]),
    /// Bencode List.
    List(&'b mut BListAccess<T>),
    /// Bencode Dictionary.
    Dict(&'b mut BDictAccess<'a, T>),
}

/// Trait for write access to some bencode type.
pub trait BMutAccess<'a>: Sized + BRefAccess<'a> {
    /// Access the bencode as a `BencodeMutKind`.
    fn kind_mut<'b>(&'b mut self) -> BencodeMutKind<'b, 'a, Self::BType>;

    /// Attempt to access the bencode as a mutable `BListAccess`.
    fn list_mut(&mut self) -> Option<&mut BListAccess<Self::BType>>;

    /// Attempt to access the bencode as a mutable `BDictAccess`.
    fn dict_mut(&mut self) -> Option<&mut BDictAccess<'a, Self::BType>>;
}