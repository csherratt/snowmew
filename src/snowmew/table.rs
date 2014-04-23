

macro_rules! table(
    ($name:ident => $value:ty) => {
        concat!("a", "b") : BTreeMap<ObjectKey, $value> 
    }
)