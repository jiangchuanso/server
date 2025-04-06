use std::ffi::{c_char, c_void, CStr, CString};
use thiserror::Error;

// Wait for core::ffi::c_size_t to be stabilized
// https://github.com/rust-lang/rust/issues/88345
use usize as c_size_t;

#[derive(Error, Debug)]
pub enum TranslatorError {
    #[error("Failed to create C string: {0}")]
    StringConversion(#[from] std::ffi::NulError),

    #[error("Translation failed: {0}")]
    TranslationFailed(String),

    #[error("Failed to create translator")]
    TranslatorCreationFailed,
}

#[link(name = "translation", kind = "static")]
unsafe extern "C" {
    fn bergamot_create(numWorkers: c_size_t) -> *mut c_void;
    fn bergamot_destroy(translator: *mut c_void);
    fn bergamot_load_model_from_config(
        translator: *mut c_void,
        languagePair: *const c_char,
        config: *const c_char,
    );
    fn bergamot_is_supported(
        translator: *mut c_void,
        from: *const c_char,
        to: *const c_char,
    ) -> bool;
    fn bergamot_translate(
        translator: *mut c_void,
        from: *const c_char,
        to: *const c_char,
        input: *const c_char,
    ) -> *const c_char;
    fn bergamot_free_translation(translation: *const c_char);
}

pub struct Translator {
    inner: *mut c_void,
}

unsafe impl Send for Translator {}
unsafe impl Sync for Translator {}

impl Translator {
    pub fn new(num_workers: usize) -> Result<Self, TranslatorError> {
        let inner = unsafe { bergamot_create(num_workers) };
        if inner.is_null() {
            return Err(TranslatorError::TranslatorCreationFailed);
        }
        Ok(Translator { inner })
    }

    pub fn load_model_from_config(
        &self,
        language_pair: &str,
        config: &str,
    ) -> Result<(), TranslatorError> {
        let language_pair_cstr = CString::new(language_pair)?;
        let config_cstr = CString::new(config)?;
        unsafe {
            bergamot_load_model_from_config(
                self.inner,
                language_pair_cstr.as_ptr(),
                config_cstr.as_ptr(),
            );
        }
        Ok(())
    }

    pub fn is_supported(&self, from_lang: &str, to_lang: &str) -> Result<bool, TranslatorError> {
        let from_cstr = CString::new(from_lang)?;
        let to_cstr = CString::new(to_lang)?;
        let supported =
            unsafe { bergamot_is_supported(self.inner, from_cstr.as_ptr(), to_cstr.as_ptr()) };
        Ok(supported)
    }

    pub fn translate(
        &self,
        from_lang: &str,
        to_lang: &str,
        input_text: &str,
    ) -> Result<String, TranslatorError> {
        let from_cstr = CString::new(from_lang)?;
        let to_cstr = CString::new(to_lang)?;
        let input_cstr = CString::new(input_text)?;
        let translated_text_ptr = unsafe {
            bergamot_translate(
                self.inner,
                from_cstr.as_ptr(),
                to_cstr.as_ptr(),
                input_cstr.as_ptr(),
            )
        };

        if translated_text_ptr.is_null() {
            return Err(TranslatorError::TranslationFailed(
                "null pointer returned".to_string(),
            ));
        }

        let translated_text_cstr = unsafe { CStr::from_ptr(translated_text_ptr) };
        let translated_text = translated_text_cstr.to_string_lossy().into_owned();

        unsafe { bergamot_free_translation(translated_text_ptr) };

        Ok(translated_text)
    }
}

impl Drop for Translator {
    fn drop(&mut self) {
        unsafe {
            bergamot_destroy(self.inner);
        }
    }
}
