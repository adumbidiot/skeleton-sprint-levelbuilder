use widestring::U16CString;
use winapi::um::shtypes::COMDLG_FILTERSPEC;

/// File type filter list
pub struct FileFilters(Vec<COMDLG_FILTERSPEC>);

impl FileFilters {
    /// Make an empty list of file type filters
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Get the number of file filters
    pub fn with_capacity(cap: usize) -> Self {
        Self(Vec::with_capacity(cap))
    }

    /// Get the number of file filters
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Get the number of file filters
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the inner COMDLG_FILTERSPEC list ptr
    pub fn as_ptr(&self) -> *const COMDLG_FILTERSPEC {
        self.0.as_ptr()
    }

    /// Add a filter.
    /// # Safety
    /// This filter must not be in use by any file dialog
    pub unsafe fn add_filter(&mut self, name: U16CString, filter: U16CString) {
        self.0.push(COMDLG_FILTERSPEC {
            pszName: name.into_raw(),
            pszSpec: filter.into_raw(),
        });
    }
}

impl Default for FileFilters {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for FileFilters {
    fn drop(&mut self) {
        while let Some(filter) = self.0.pop() {
            unsafe {
                drop(U16CString::from_raw(filter.pszName as *mut u16));
                drop(U16CString::from_raw(filter.pszSpec as *mut u16));
            }
        }
    }
}
