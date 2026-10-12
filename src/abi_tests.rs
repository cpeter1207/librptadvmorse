use super::*;

#[test]
fn descriptor_uses_status_and_opaque_output_handle() {
    let api = unsafe { &*rptadv_morse_descriptor() };
    assert_eq!(api.struct_size as usize, size_of::<MorseDescriptor>());
    assert_eq!(api.abi_version, 1);
    assert_eq!(
        unsafe { std::ffi::CStr::from_ptr(api.capability_name) }.to_bytes(),
        b"rptadv.morse"
    );

    let mut handle = ptr::null_mut();
    let text = b"E T";
    unsafe {
        assert_eq!(
            (api.create)(text.as_ptr(), text.len(), 20, 1_000.0, -6, &mut handle),
            0
        );
        assert!(!handle.is_null());
        let mut pcm = [99.0; 7];
        let mut written = 0;
        assert_eq!(
            (api.render)(handle, pcm.as_mut_ptr(), pcm.len(), &mut written),
            0
        );
        assert_eq!(written, pcm.len());
        assert_eq!((api.restart)(handle), 0);
        let mut replay = [99.0; 7];
        assert_eq!(
            (api.render)(handle, replay.as_mut_ptr(), replay.len(), &mut written),
            0
        );
        assert_eq!(pcm.map(f32::to_bits), replay.map(f32::to_bits));
        assert_eq!((api.render)(handle, ptr::null_mut(), 0, &mut written), 0);
        assert_eq!(written, 0);
        assert_eq!((api.render)(handle, ptr::null_mut(), 1, &mut written), 1);
        assert_eq!(written, 0);
        assert_eq!(
            (api.render)(handle, pcm.as_mut_ptr(), 1, ptr::null_mut()),
            1
        );
        (api.destroy)(handle);

        assert_eq!(
            (api.create)(ptr::null(), 0, 20, 1_000.0, -6, &mut handle),
            0
        );
        assert!(!handle.is_null());
        (api.destroy)(handle);

        assert_eq!(
            (api.create)(text.as_ptr(), text.len(), 20, 1_000.0, -6, ptr::null_mut()),
            1
        );
        assert_eq!(
            (api.create)(ptr::null(), 1, 20, 1_000.0, -6, &mut handle),
            1
        );
        assert!(handle.is_null());
        assert_eq!(
            (api.create)([0xff].as_ptr(), 1, 20, 1_000.0, -6, &mut handle),
            1
        );
        assert_eq!(
            (api.create)(b"*".as_ptr(), 1, 20, 1_000.0, -6, &mut handle),
            1
        );
        assert_eq!(
            (api.create)(b"E".as_ptr(), 1, 0, 1_000.0, -6, &mut handle),
            1
        );
        assert_eq!(
            (api.render)(ptr::null_mut(), ptr::null_mut(), 0, &mut written),
            1
        );
        assert_eq!((api.restart)(ptr::null_mut()), 1);
        (api.destroy)(ptr::null_mut());
    }
}
