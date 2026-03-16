// SAFETY: The C++ DeckLink bridge types use internal mutexes and are safe to send
// across threads. DeckLinkInputCapture uses std::mutex + condition_variable,
// DeckLinkOutputPlayer uses frame pools with synchronous calls, and
// DeckLinkSystem is read-only after construction.
unsafe impl Send for decklink_ffi::DeckLinkSystem {}
unsafe impl Send for decklink_ffi::DeckLinkInputCapture {}
unsafe impl Send for decklink_ffi::DeckLinkOutputPlayer {}

#[allow(dead_code)]
#[cxx::bridge(namespace = "momo")]
pub mod decklink_ffi {
    struct BridgeDeviceInfo {
        pub index: u32,
        pub name: String,
        pub model_name: String,
        pub has_input: bool,
        pub has_output: bool,
    }

    unsafe extern "C++" {
        include!("decklink_bridge.h");

        type DeckLinkSystem;
        fn create_system() -> UniquePtr<DeckLinkSystem>;
        fn is_api_present(self: &DeckLinkSystem) -> bool;
        fn enumerate(self: &DeckLinkSystem) -> Vec<BridgeDeviceInfo>;

        type DeckLinkInputCapture;
        fn create_capture(
            sys: &DeckLinkSystem,
            device_idx: u32,
            mode: u32,
            fmt: u32,
        ) -> UniquePtr<DeckLinkInputCapture>;
        fn start(self: Pin<&mut DeckLinkInputCapture>) -> bool;
        fn stop(self: Pin<&mut DeckLinkInputCapture>);
        fn get_frame(self: Pin<&mut DeckLinkInputCapture>, timeout_ms: u32) -> Vec<u8>;
        fn frame_width(self: &DeckLinkInputCapture) -> u32;
        fn frame_height(self: &DeckLinkInputCapture) -> u32;

        type DeckLinkOutputPlayer;
        fn create_player(
            sys: &DeckLinkSystem,
            device_idx: u32,
            mode: u32,
            fmt: u32,
        ) -> UniquePtr<DeckLinkOutputPlayer>;
        fn start(self: Pin<&mut DeckLinkOutputPlayer>) -> bool;
        fn stop(self: Pin<&mut DeckLinkOutputPlayer>);
        fn send_frame(self: Pin<&mut DeckLinkOutputPlayer>, data: &[u8]) -> bool;
    }
}
