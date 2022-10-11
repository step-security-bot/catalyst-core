use chain_core::{
    packer::Codec,
    property::{DeserializeFromSlice, Serialize},
};

/// test that any arbitrary given object can serialize and deserialize
/// back into itself (i.e. it is a bijection,  or a one to one match
/// between the serialized bytes and the object)
pub fn serialization_bijection<T>(t: T)
where
    T: std::fmt::Debug + Serialize + DeserializeFromSlice + Eq,
{
    let vec = t.serialize_as_vec().unwrap();
    let decoded_t = T::deserialize_from_slice(&mut Codec::new(&vec)).unwrap();

    assert_eq!(vec.len(), t.serialized_size());
    assert_eq!(decoded_t, t);
}
