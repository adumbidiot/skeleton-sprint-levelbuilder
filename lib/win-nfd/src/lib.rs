mod file_filters;
mod sys;

pub use crate::file_filters::FileFilters;
use crate::sys::{
    IFileOpenDialog,
    IFileSaveDialog,
    IShellItem,
    CLSID_FILEOPENDIALOG,
    CLSID_FILESAVEDIALOG,
    IID_ISHELL_ITEM,
};
use com::sys::FAILED;
use std::{
    cell::RefCell,
    ffi::{
        OsStr,
        OsString,
    },
    mem::MaybeUninit,
    os::windows::ffi::OsStringExt,
    path::{
        Path,
        PathBuf,
    },
};
use widestring::U16CString;
use winapi::{
    ctypes::c_void,
    shared::windef::HWND,
    um::{
        combaseapi::CoTaskMemFree,
        fileapi::GetFullPathNameW,
        shobjidl_core::{
            SHCreateItemFromParsingName,
            SIGDN,
            SIGDN_DESKTOPABSOLUTEEDITING,
            SIGDN_DESKTOPABSOLUTEPARSING,
            SIGDN_FILESYSPATH,
            SIGDN_NORMALDISPLAY,
            SIGDN_PARENTRELATIVE,
            SIGDN_PARENTRELATIVEEDITING,
            SIGDN_PARENTRELATIVEFORADDRESSBAR,
            SIGDN_PARENTRELATIVEFORUI,
            SIGDN_PARENTRELATIVEPARSING,
            SIGDN_URL,
        },
        winbase::lstrlenW,
    },
};

#[derive(Debug, thiserror::Error)]
pub enum NfdError {
    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    NulError(#[from] widestring::NulError<u16>),
}

/// File Open dialog
pub struct FileOpenDialog {
    ptr: IFileOpenDialog,

    // TODO: Consider ArcSwap or similar instead of RefCell
    /// IFileOpenDialog doesn't own its file filters.
    file_filters: RefCell<FileFilters>,
}

impl FileOpenDialog {
    /// Try to make a new open file dialog
    pub fn new() -> Result<Self, NfdError> {
        let ptr = com::runtime::create_instance::<IFileOpenDialog>(&CLSID_FILEOPENDIALOG)
            .map_err(std::io::Error::from_raw_os_error)?;

        Ok(Self {
            ptr,
            file_filters: RefCell::new(FileFilters::new()),
        })
    }

    /// Show the window
    pub fn show(&self, parent: Option<HWND>) -> Result<(), NfdError> {
        let ret = unsafe { self.ptr.Show(parent.unwrap_or(std::ptr::null_mut())) };

        if FAILED(ret) {
            Err(NfdError::Io(std::io::Error::from_raw_os_error(ret)))
        } else {
            Ok(())
        }
    }

    /// Set the default folder
    pub fn set_default_folder(&self, item: ShellItem) -> Result<(), NfdError> {
        let ret = unsafe { self.ptr.SetDefaultFolder(item.0) };

        if FAILED(ret) {
            Err(NfdError::Io(std::io::Error::from_raw_os_error(ret)))
        } else {
            Ok(())
        }
    }

    /// Set the folder to open
    pub fn set_folder(&self, item: ShellItem) -> Result<(), NfdError> {
        let ret = unsafe { self.ptr.SetFolder(item.0) };

        if FAILED(ret) {
            Err(NfdError::Io(std::io::Error::from_raw_os_error(ret)))
        } else {
            Ok(())
        }
    }

    /// Set the file types
    pub fn set_filetypes(&self, filters: &[(&OsStr, &OsStr)]) -> Result<(), NfdError> {
        let filters = {
            let mut file_filters = FileFilters::with_capacity(filters.len());
            for filter in filters {
                let name = U16CString::from_os_str(filter.0)?;
                let filter = U16CString::from_os_str(filter.1)?;
                unsafe {
                    file_filters.add_filter(name, filter);
                }
            }
            file_filters
        };

        let ret = unsafe {
            self.ptr
                .SetFileTypes(filters.len() as u32, filters.as_ptr())
        };

        if FAILED(ret) {
            Err(NfdError::Io(std::io::Error::from_raw_os_error(ret)))
        } else {
            *self.file_filters.borrow_mut() = filters;

            Ok(())
        }
    }

    /// Set filename
    pub fn set_filename(&self, filename: &OsStr) -> Result<(), NfdError> {
        let filename = U16CString::from_os_str(filename)?;

        let ret = unsafe { self.ptr.SetFileName(filename.as_ptr()) };

        if FAILED(ret) {
            Err(NfdError::Io(std::io::Error::from_raw_os_error(ret)))
        } else {
            Ok(())
        }
    }

    /// Get single result
    pub fn get_result(&self) -> Result<ShellItem, NfdError> {
        let mut shell = MaybeUninit::zeroed();
        let ret = unsafe { self.ptr.GetResult(shell.as_mut_ptr()) };

        if FAILED(ret) {
            Err(NfdError::Io(std::io::Error::from_raw_os_error(ret)))
        } else {
            Ok(unsafe { shell.assume_init() }.into())
        }
    }
}

/// Display type for shellitem
/// Requests the form of an item's display name to retrieve through IShellItem::GetDisplayName and SHGetNameFromIDList.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum DisplayType {
    /// Returns the display name relative to the parent folder. In UI this name is generally ideal for display to the user.
    NormalDisplay,

    /// Returns the parsing name relative to the parent folder. This name is not suitable for use in UI.
    ParentRelativeParsing,

    /// Returns the parsing name relative to the desktop. This name is not suitable for use in UI.
    DesktopAbsoluteParsing,

    /// Returns the editing name relative to the parent folder. In UI this name is suitable for display to the user.
    ParentRelativeEditing,

    /// Returns the editing name relative to the desktop. In UI this name is suitable for display to the user.
    DesktopAbsoluteEditing,

    /// Returns the item's file system path, if it has one.
    /// Only items that report SFGAO_FILESYSTEM have a file system path.
    /// When an item does not have a file system path, a call to IShellItem::GetDisplayName on that item will fail.
    /// In UI this name is suitable for display to the user in some cases, but note that it might not be specified for all items.
    FileSysPath,

    /// Returns the item's URL, if it has one.
    /// Some items do not have a URL, and in those cases a call to IShellItem::GetDisplayName will fail.
    /// This name is suitable for display to the user in some cases, but note that it might not be specified for all items.
    Url,

    /// Returns the path relative to the parent folder in a friendly format as displayed in an address bar.
    /// This name is suitable for display to the user.
    ParentRelativeForAddressBar,

    /// Returns the path relative to the parent folder.
    ParentRelative,

    /// Introduced in Windows 8.
    ParentRelativeForUi,
}

impl Into<SIGDN> for DisplayType {
    fn into(self) -> SIGDN {
        match self {
            Self::NormalDisplay => SIGDN_NORMALDISPLAY,
            Self::ParentRelativeParsing => SIGDN_PARENTRELATIVEPARSING,
            Self::DesktopAbsoluteParsing => SIGDN_DESKTOPABSOLUTEPARSING,
            Self::ParentRelativeEditing => SIGDN_PARENTRELATIVEEDITING,
            Self::DesktopAbsoluteEditing => SIGDN_DESKTOPABSOLUTEEDITING,
            Self::FileSysPath => SIGDN_FILESYSPATH,
            Self::Url => SIGDN_URL,
            Self::ParentRelativeForAddressBar => SIGDN_PARENTRELATIVEFORADDRESSBAR,
            Self::ParentRelative => SIGDN_PARENTRELATIVE,
            Self::ParentRelativeForUi => SIGDN_PARENTRELATIVEFORUI,
        }
    }
}

/// Shell Item
#[repr(transparent)]
pub struct ShellItem(IShellItem);

impl ShellItem {
    /// Make a shellitem from a path
    pub fn from_path(path: &Path) -> Result<Self, NfdError> {
        let path = U16CString::from_os_str(path.as_os_str())?;

        let mut shell = MaybeUninit::zeroed();
        let buf = &mut [0; 4096];
        let ret = unsafe {
            let path_len = GetFullPathNameW(
                path.as_ptr(),
                buf.len() as u32,
                buf.as_mut_ptr(),
                std::ptr::null_mut(),
            );

            if path_len == 0 {
                return Err(NfdError::Io(std::io::Error::last_os_error()));
            }

            SHCreateItemFromParsingName(
                buf.as_ptr(),
                std::ptr::null_mut(),
                (&IID_ISHELL_ITEM as *const com::sys::GUID).cast(),
                shell.as_mut_ptr(),
            )
        };

        if FAILED(ret) {
            Err(NfdError::Io(std::io::Error::from_raw_os_error(ret)))
        } else {
            Ok(Self(unsafe { std::mem::transmute(shell.assume_init()) }))
        }
    }

    /// Get the display name of a shellitem
    pub fn get_display_name(&self, display_type: DisplayType) -> Result<OsString, NfdError> {
        let display_type: SIGDN = display_type.into();
        let mut wstr = std::ptr::null_mut();
        let ret = unsafe { self.0.GetDisplayName(display_type, &mut wstr) };

        if FAILED(ret) {
            Err(NfdError::Io(std::io::Error::from_raw_os_error(ret)))
        } else {
            Ok(unsafe {
                let len = lstrlenW(wstr);
                let os_str = OsString::from_wide(std::slice::from_raw_parts(wstr, len as usize));
                CoTaskMemFree(wstr as *mut c_void);

                os_str
            })
        }
    }

    /// Get the shellitem's path
    pub fn get_filesystem_path(&self) -> Result<PathBuf, NfdError> {
        Ok(self.get_display_name(DisplayType::FileSysPath)?.into())
    }
}

impl From<IShellItem> for ShellItem {
    fn from(item: IShellItem) -> Self {
        ShellItem(item)
    }
}

impl std::fmt::Debug for ShellItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShellItem")
            .field("filesystem_path", &self.get_filesystem_path())
            .finish()
    }
}

/// Builder for a FileOpenDialog
pub struct FileOpenDialogBuilder<'a> {
    /// Whether to init com
    pub init_com: bool,

    /// Path to open by default
    pub default_path: Option<&'a Path>,

    /// Path to open, regardless of past choices
    pub path: Option<&'a Path>,

    /// File types
    pub filetypes: Vec<(&'a OsStr, &'a OsStr)>,

    /// Filename
    pub filename: Option<&'a OsStr>,
}

impl<'a> FileOpenDialogBuilder<'a> {
    /// Make a new FileOpenDialogBuilder
    pub fn new() -> Self {
        FileOpenDialogBuilder {
            init_com: false,
            default_path: None,
            path: None,
            filetypes: Vec::new(),
            filename: None,
        }
    }

    /// Whether to init com
    pub fn init_com(&mut self) -> &mut Self {
        self.init_com = true;
        self
    }

    /// Set the default path where the dialog will open
    pub fn default_path(&mut self, default_path: &'a Path) -> &mut Self {
        self.default_path = Some(default_path);
        self
    }

    /// Set the path where the dialog will open
    pub fn path(&mut self, path: &'a Path) -> &mut Self {
        self.path = Some(path);
        self
    }

    /// Add a file type
    pub fn filetype(&mut self, name: &'a OsStr, filter: &'a OsStr) -> &mut Self {
        self.filetypes.push((name, filter));
        self
    }

    /// Set the default filename
    pub fn filename(&mut self, filename: &'a OsStr) -> &mut Self {
        self.filename = Some(filename);
        self
    }

    /// Build a dialog.
    pub fn build(&self) -> Result<FileOpenDialog, NfdError> {
        if self.init_com {
            com::runtime::init_runtime().map_err(std::io::Error::from_raw_os_error)?;
        }

        let dialog = FileOpenDialog::new()?;

        if let Some(default_path) = self.default_path {
            let shell_item = ShellItem::from_path(default_path)?;
            dialog.set_default_folder(shell_item)?;
        }

        if let Some(path) = self.path {
            let shell_item = ShellItem::from_path(&path)?;
            dialog.set_folder(shell_item)?;
        }

        if !self.filetypes.is_empty() {
            dialog.set_filetypes(&self.filetypes)?;
        }

        if let Some(filename) = self.filename {
            dialog.set_filename(filename)?;
        }

        Ok(dialog)
    }

    /// Execute a dialog.
    pub fn execute(&self) -> Result<PathBuf, NfdError> {
        let dialog = self.build()?;

        dialog.show(None)?;
        let shellitem = dialog.get_result()?;

        Ok(shellitem.get_filesystem_path()?)
    }
}

impl Default for FileOpenDialogBuilder<'_> {
    fn default() -> Self {
        FileOpenDialogBuilder::new()
    }
}

/// File Save dialog
pub struct FileSaveDialog {
    ptr: IFileSaveDialog,

    // TODO: Consider ArcSwap or similar instead of RefCell
    /// IFileOpenDialog doesn't own its file filters.
    file_filters: RefCell<FileFilters>,
}

impl FileSaveDialog {
    /// Try to make a new open file save dialog
    pub fn new() -> Result<Self, NfdError> {
        let ptr = com::runtime::create_instance::<IFileSaveDialog>(&CLSID_FILESAVEDIALOG)
            .map_err(std::io::Error::from_raw_os_error)?;

        Ok(Self {
            ptr,
            file_filters: RefCell::new(FileFilters::new()),
        })
    }

    /// Show the window
    pub fn show(&self, parent: Option<HWND>) -> Result<(), NfdError> {
        let ret = unsafe { self.ptr.Show(parent.unwrap_or(std::ptr::null_mut())) };

        if FAILED(ret) {
            Err(NfdError::Io(std::io::Error::from_raw_os_error(ret)))
        } else {
            Ok(())
        }
    }

    /// Set the default folder
    pub fn set_default_folder(&self, item: ShellItem) -> Result<(), NfdError> {
        let ret = unsafe { self.ptr.SetDefaultFolder(item.0) };

        if FAILED(ret) {
            Err(NfdError::Io(std::io::Error::from_raw_os_error(ret)))
        } else {
            Ok(())
        }
    }

    /// Set the folder to open
    pub fn set_folder(&self, item: ShellItem) -> Result<(), NfdError> {
        let ret = unsafe { self.ptr.SetFolder(item.0) };

        if FAILED(ret) {
            Err(NfdError::Io(std::io::Error::from_raw_os_error(ret)))
        } else {
            Ok(())
        }
    }

    /// Set the file types
    pub fn set_filetypes(&self, filters: &[(&OsStr, &OsStr)]) -> Result<(), NfdError> {
        let filters = {
            let mut file_filters = FileFilters::with_capacity(filters.len());
            for filter in filters {
                let name = U16CString::from_os_str(filter.0)?;
                let filter = U16CString::from_os_str(filter.1)?;
                unsafe {
                    file_filters.add_filter(name, filter);
                }
            }
            file_filters
        };

        let ret = unsafe {
            self.ptr
                .SetFileTypes(filters.len() as u32, filters.as_ptr())
        };

        if FAILED(ret) {
            Err(NfdError::Io(std::io::Error::from_raw_os_error(ret)))
        } else {
            *self.file_filters.borrow_mut() = filters;

            Ok(())
        }
    }

    /// Set filename
    pub fn set_filename(&self, filename: &OsStr) -> Result<(), NfdError> {
        let filename = U16CString::from_os_str(filename)?;

        let ret = unsafe { self.ptr.SetFileName(filename.as_ptr()) };

        if FAILED(ret) {
            Err(NfdError::Io(std::io::Error::from_raw_os_error(ret)))
        } else {
            Ok(())
        }
    }

    /// Get single result
    pub fn get_result(&self) -> Result<ShellItem, NfdError> {
        let mut shell = MaybeUninit::zeroed();
        let ret = unsafe { self.ptr.GetResult(shell.as_mut_ptr()) };

        if FAILED(ret) {
            Err(NfdError::Io(std::io::Error::from_raw_os_error(ret)))
        } else {
            Ok(unsafe { shell.assume_init() }.into())
        }
    }
}

/// Builder for a FileSaveDialog
pub struct FileSaveDialogBuilder<'a> {
    /// Whether to init com
    pub init_com: bool,

    /// Path to open by default
    pub default_path: Option<&'a Path>,

    /// Path to open, regardless of past choices
    pub path: Option<&'a Path>,

    /// File types
    pub filetypes: Vec<(&'a OsStr, &'a OsStr)>,

    /// Filename
    pub filename: Option<&'a OsStr>,
}

impl<'a> FileSaveDialogBuilder<'a> {
    /// Make a new FileSaveDialogBuilder
    pub fn new() -> Self {
        FileSaveDialogBuilder {
            init_com: false,
            default_path: None,
            path: None,
            filetypes: Vec::new(),
            filename: None,
        }
    }

    /// Whether to init com
    pub fn init_com(&mut self) -> &mut Self {
        self.init_com = true;
        self
    }

    /// Set the default path where the dialog will open
    pub fn default_path(&mut self, default_path: &'a Path) -> &mut Self {
        self.default_path = Some(default_path);
        self
    }

    /// Set the path where the dialog will open
    pub fn path(&mut self, path: &'a Path) -> &mut Self {
        self.path = Some(path);
        self
    }

    /// Add a file type
    pub fn filetype(&mut self, name: &'a OsStr, filter: &'a OsStr) -> &mut Self {
        self.filetypes.push((name, filter));
        self
    }

    /// Set the default filename
    pub fn filename(&mut self, filename: &'a OsStr) -> &mut Self {
        self.filename = Some(filename);
        self
    }

    /// Build a dialog.
    pub fn build(&self) -> Result<FileSaveDialog, NfdError> {
        if self.init_com {
            com::runtime::init_runtime().map_err(std::io::Error::from_raw_os_error)?;
        }

        let dialog = FileSaveDialog::new()?;

        if let Some(default_path) = self.default_path {
            let shell_item = ShellItem::from_path(default_path)?;
            dialog.set_default_folder(shell_item)?;
        }

        if let Some(path) = self.path {
            let shell_item = ShellItem::from_path(&path)?;
            dialog.set_folder(shell_item)?;
        }

        if !self.filetypes.is_empty() {
            dialog.set_filetypes(&self.filetypes)?;
        }

        if let Some(filename) = self.filename {
            dialog.set_filename(filename)?;
        }

        Ok(dialog)
    }

    /// Execute a dialog.
    pub fn execute(&self) -> Result<PathBuf, NfdError> {
        let dialog = self.build()?;

        dialog.show(None)?;
        let shellitem = dialog.get_result()?;

        Ok(shellitem.get_filesystem_path()?)
    }
}

impl Default for FileSaveDialogBuilder<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// Default nfd open dialog.
/// Look at this functions impl and write your own if you need more control
pub fn nfd_open() -> Result<PathBuf, NfdError> {
    Ok(FileOpenDialogBuilder::new().init_com().execute()?)
}

/// Default nfd save dialog.
/// Look at this functions impl and write your own if you need more control
pub fn nfd_save() -> Result<PathBuf, NfdError> {
    Ok(FileSaveDialogBuilder::new().init_com().execute()?)
}

/// Shothand for `FileOpenDialogBuilder::new().init_com()`
pub fn nfd_open_builder() -> FileOpenDialogBuilder<'static> {
    let mut builder = FileOpenDialogBuilder::new();
    builder.init_com();
    builder
}

/// Shothand for `FileSaveDialogBuilder::new().init_com()`
pub fn nfd_save_builder() -> FileSaveDialogBuilder<'static> {
    let mut builder = FileSaveDialogBuilder::new();
    builder.init_com();
    builder
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;
    use winapi::um::shellscalingapi::{
        SetProcessDpiAwareness,
        PROCESS_PER_MONITOR_DPI_AWARE,
    };

    /// Make the dialog window dpi aware
    fn set_dpi() {
        static SET_DPI: Once = Once::new();
        unsafe {
            SET_DPI.call_once(|| {
                SetProcessDpiAwareness(PROCESS_PER_MONITOR_DPI_AWARE);
            });
        }
    }

    #[test]
    #[ignore]
    fn it_works_open_default() {
        set_dpi();

        println!(
            "Open File Path (nfd): {}",
            nfd_open().expect("nfd").display()
        );
    }

    #[test]
    #[ignore]
    fn it_works_open() {
        set_dpi();

        let path = FileOpenDialogBuilder::new()
            .init_com()
            .default_path(".".as_ref())
            .path(".".as_ref())
            .filetype("toml".as_ref(), "*.toml".as_ref())
            .filetype("sks".as_ref(), "*.txt;*.lbl".as_ref())
            .execute()
            .expect("File dialog exec");

        println!("Open File Path (builder): {}", path.display());
    }

    #[test]
    #[ignore]
    fn it_works_save_default() {
        set_dpi();

        println!(
            "Save File Path (nfd): {}",
            nfd_open().expect("nfd").display()
        );
    }

    #[test]
    #[ignore]
    fn it_works_save() {
        set_dpi();

        let path = FileSaveDialogBuilder::new()
            .init_com()
            .default_path(".".as_ref())
            .path(".".as_ref())
            .filetype("toml".as_ref(), "*.toml".as_ref())
            .filetype("sks".as_ref(), "*.txt;*.lbl".as_ref())
            .filename("level.txt".as_ref())
            .execute()
            .expect("File dialog exec");

        println!("Save File Path (builder): {}", path.display());
    }
}
