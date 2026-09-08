cfg_if::cfg_if! {
    if #[cfg(unix)] {
        use crate::secret::SecretThing;
    }
}
