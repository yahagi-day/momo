#pragma once

#include "rust/cxx.h"
#include <cstdint>
#include <memory>
#include <mutex>
#include <condition_variable>
#include <vector>
#include <atomic>
#include <string>

// Forward declarations for DeckLink SDK types
class IDeckLink;
class IDeckLinkIterator;
class IDeckLinkInput;
class IDeckLinkOutput;
class IDeckLinkInputCallback;
class IDeckLinkVideoInputFrame;
class IDeckLinkAudioInputPacket;
class IDeckLinkDisplayMode;
class IDeckLinkMutableVideoFrame;

namespace momo {

struct BridgeDeviceInfo;

class DeckLinkSystem {
public:
    DeckLinkSystem();
    ~DeckLinkSystem();

    bool is_api_present() const;
    rust::Vec<BridgeDeviceInfo> enumerate() const;

private:
    bool api_present_;
};

class InputCallbackImpl;

class DeckLinkInputCapture {
public:
    DeckLinkInputCapture(IDeckLinkInput* input, uint32_t width, uint32_t height);
    ~DeckLinkInputCapture();

    bool start();
    void stop();
    rust::Vec<uint8_t> get_frame(uint32_t timeout_ms);
    uint32_t frame_width() const;
    uint32_t frame_height() const;

    // Called by InputCallbackImpl
    void on_frame(const void* data, size_t size);

private:
    IDeckLinkInput* input_;
    InputCallbackImpl* callback_;
    uint32_t width_;
    uint32_t height_;
    bool running_;

    std::mutex mutex_;
    std::condition_variable cv_;
    std::vector<uint8_t> buffer_;
    bool frame_ready_;
};

class DeckLinkOutputPlayer {
public:
    DeckLinkOutputPlayer(IDeckLinkOutput* output, uint32_t mode, uint32_t pixel_format,
                         int32_t width, int32_t height, int32_t row_bytes);
    ~DeckLinkOutputPlayer();

    bool start();
    void stop();
    bool send_frame(rust::Slice<const uint8_t> data);

private:
    IDeckLinkOutput* output_;
    uint32_t mode_;
    uint32_t pixel_format_;
    int32_t width_;
    int32_t height_;
    int32_t row_bytes_;
    bool running_;

    static constexpr int FRAME_POOL_SIZE = 3;
    IDeckLinkMutableVideoFrame* frame_pool_[FRAME_POOL_SIZE];
    int current_frame_;
};

// Factory functions called from Rust via cxx
std::unique_ptr<DeckLinkSystem> create_system();
std::unique_ptr<DeckLinkInputCapture> create_capture(
    const DeckLinkSystem& sys, uint32_t device_idx, uint32_t mode, uint32_t fmt);
std::unique_ptr<DeckLinkOutputPlayer> create_player(
    const DeckLinkSystem& sys, uint32_t device_idx, uint32_t mode, uint32_t fmt);

} // namespace momo
