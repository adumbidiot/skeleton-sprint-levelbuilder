#![allow(clippy::transmute_ptr_to_ptr)]
use com::{
    interfaces::IUnknown,
    sys::{
        BOOL,
        GUID,
        HRESULT,
    },
};
use winapi::{
    ctypes::{
        c_int,
        c_void,
    },
    shared::{
        guiddef::{
            REFGUID,
            REFIID,
        },
        minwindef::{
            DWORD,
            UINT,
        },
        windef::HWND,
    },
    um::{
        objidl::IBindCtx,
        propsys::{
            IPropertyDescriptionList,
            IPropertyStore,
        },
        shobjidl_core::{
            IShellItemArray,
            IShellItemFilter,
            SFGAOF,
            SICHINTF,
            SIGDN,
        },
        shtypes::COMDLG_FILTERSPEC,
        winnt::{
            LPCWSTR,
            LPWSTR,
        },
    },
};

pub const CLSID_FILEOPENDIALOG: GUID = GUID {
    data1: 0xDC1C5A9C,
    data2: 0xE88A,
    data3: 0x4dde,
    data4: 0xA5A160F82A20AEF7_u64.to_be_bytes(),
};

pub const CLSID_FILESAVEDIALOG: GUID = GUID {
    data1: 0xC0B4E2F3,
    data2: 0xBA21,
    data3: 0x4773,
    data4: 0x8DBA335EC946EB8B_u64.to_be_bytes(),
};

com::interfaces! {
    #[uuid("B4DB1657-70D7-485E-8E3E-6FCB5A5C1802")]
    pub unsafe interface IModalWindow: IUnknown {
         pub fn Show(&self, hwndOwner: HWND) -> HRESULT;
    }

    #[uuid("42F85136-DB7E-439C-85F1-E4075D135FC8")]
    pub unsafe interface IFileDialog: IModalWindow {
        pub fn SetFileTypes(&self, cFileTypes: UINT, rgFilterSpec: *const COMDLG_FILTERSPEC) -> HRESULT;
        fn SetFileTypeIndex(&self, iFileType: UINT) -> HRESULT;
        fn GetFileTypeIndex(&self, piFileType: *mut UINT) -> HRESULT;
        // Missing: IFileDialogEvents
        fn Advise(&self, pfde: *mut c_void, pdwCookie: *mut DWORD) -> HRESULT;
        fn Unadvise(&self, dwCookie: DWORD) -> HRESULT;
        // Missing: FILEOPENDIALOGOPTIONS (SIZED TYPE MISSING, DO NOT USE!!!!!!)
        fn SetOptions(&self, fos: u8) -> HRESULT;
        // Missing: FILEOPENDIALOGOPTIONS (SIZED TYPE MISSING, DO NOT USE!!!!!!)
        fn GetOptions(&self, pfos: *mut u8) -> HRESULT;
        pub fn SetDefaultFolder(&self, psi: IShellItem) -> HRESULT;
        pub fn SetFolder(&self, psi: IShellItem) -> HRESULT;
        fn GetFolder(&self, ppsi: *mut IShellItem) -> HRESULT;
        fn GetCurrentSelection(&self, ppsi: *mut IShellItem) -> HRESULT;
        pub fn SetFileName(&self, pszName: LPCWSTR)-> HRESULT;
        fn GetFileName(&self, pszName: *mut LPWSTR) -> HRESULT;
        fn SetTitle(&self, pszTitle: LPCWSTR) -> HRESULT;
        fn SetOkButtonLabel(&self, pszText: LPCWSTR) -> HRESULT;
        fn SetFileNameLabel(&self, pszLabel: LPCWSTR) -> HRESULT;
        pub fn GetResult(&self, ppsi: *mut IShellItem) -> HRESULT;
        // Missing: FDAP (SIZED TYPE MISSING, DO NOT USE!!!!!!)
        fn AddPlace(&self, psi: IShellItem, fdap: u8) -> HRESULT;
        fn SetDefaultExtension(&self, pszDefaultExtension: LPCWSTR) -> HRESULT;
        fn Close(&self, hr: HRESULT) -> HRESULT;
        fn SetClientGuid(&self, guid: REFGUID) -> HRESULT;
        fn ClearClientData(&self) -> HRESULT;
        fn SetFilter(&self, pFilter: *mut IShellItemFilter) -> HRESULT;
    }

    #[uuid("d57c7288-d4ad-4768-be02-9d969532d960")]
    pub unsafe interface IFileOpenDialog: IFileDialog {
        fn GetResults(&self, ppenum: *mut *mut IShellItemArray) -> HRESULT;
        fn GetSelectedItems(&self, ppsai: *mut *mut IShellItemArray) -> HRESULT;
    }

    #[uuid("43826d1e-e718-42ee-bc55-a1e261c37bfe")]
    pub unsafe interface IShellItem: IUnknown {
        fn BindToHandler(&self, pbc: *mut IBindCtx , bhid: REFGUID, riid: REFIID , ppv: *mut *mut c_void) -> HRESULT;
        fn GetParent(&self, ppsi: *mut IShellItem) -> HRESULT;
        pub fn GetDisplayName(&self, sigdnName: SIGDN, ppszName: *mut LPWSTR) -> HRESULT;
        fn GetAttributes(&self, sfgaoMask: SFGAOF, psfgaoAttribs: *mut SFGAOF) -> HRESULT;
        fn Compare(&self, psi: IShellItem, hint: SICHINTF, piOrder: *mut c_int) -> HRESULT;
    }

    #[uuid("84bccd23-5fde-4cdb-aea4-af64b83d78ab")]
    pub unsafe interface IFileSaveDialog: IFileDialog {
        fn SetSaveAsItem(&self, psi: IShellItem) -> HRESULT;
        fn SetProperties(&self, pStore: *mut IPropertyStore) -> HRESULT;
        fn SetCollectedProperties(&self, pList: *mut IPropertyDescriptionList, fAppendDefault: BOOL) -> HRESULT;
        fn GetProperties(&self, ppStore: *mut *mut IPropertyStore) -> HRESULT;
        // Missing: IFileOperationProgressSink
        fn ApplyProperties(&self, psi: IShellItem, pStore: *mut IPropertyStore, hwnd: HWND, pSink: *mut c_void) -> HRESULT;
    }
}
