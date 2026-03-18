#include "decklink_bridge.h"
#include "momo-decklink/src/ffi.rs.h"

#ifdef _WIN32
#include <combaseapi.h>
#include "DeckLinkAPI_h.h"
#else
#include "DeckLinkAPI.h"
#endif

#include <cstring>
#include <chrono>

// --- Platform helpers ---

#ifdef _WIN32

static IDeckLinkIterator* CreateDeckLinkIteratorInstance() {
    IDeckLinkIterator* iter = nullptr;
    HRESULT hr = CoCreateInstance(
        CLSID_CDeckLinkIterator, nullptr, CLSCTX_ALL,
        IID_IDeckLinkIterator, (void**)&iter);
    return SUCCEEDED(hr) ? iter : nullptr;
}

static bool IsDeckLinkAPIPresent() {
    IDeckLinkIterator* iter = CreateDeckLinkIteratorInstance();
    if (iter) { iter->Release(); return true; }
    return false;
}

static rust::String decklink_string_to_rust(BSTR bstr) {
    if (!bstr) return rust::String("Unknown");
    int len = WideCharToMultiByte(CP_UTF8, 0, bstr, -1, nullptr, 0, nullptr, nullptr);
    if (len <= 0) return rust::String("Unknown");
    std::string utf8(len - 1, '\0');
    WideCharToMultiByte(CP_UTF8, 0, bstr, -1, &utf8[0], len, nullptr, nullptr);
    return rust::String(utf8);
}

#else

// Linux: functions provided by DeckLinkAPIDispatch.cpp
extern "C" {
    IDeckLinkIterator* CreateDeckLinkIteratorInstance(void);
    bool IsDeckLinkAPIPresent(void);
}

static rust::String decklink_string_to_rust(const char* str) {
    if (!str) return rust::String("Unknown");
    rust::String result(str);
    free(const_cast<char*>(str));
    return result;
}

#endif

namespace momo {

// --- InputCallbackImpl ---

class InputCallbackImpl : public IDeckLinkInputCallback {
public:
    InputCallbackImpl(DeckLinkInputCapture* owner)
        : owner_(owner), ref_count_(1) {}

    // IUnknown
    HRESULT STDMETHODCALLTYPE QueryInterface(REFIID, LPVOID*) override {
        return E_NOINTERFACE;
    }
    ULONG STDMETHODCALLTYPE AddRef() override {
#ifdef _WIN32
        return InterlockedIncrement(&ref_count_);
#else
        return __sync_add_and_fetch(&ref_count_, 1);
#endif
    }
    ULONG STDMETHODCALLTYPE Release() override {
#ifdef _WIN32
        LONG count = InterlockedDecrement(&ref_count_);
#else
        ULONG count = __sync_sub_and_fetch(&ref_count_, 1);
#endif
        if (count == 0) delete this;
        return count;
    }

    // IDeckLinkInputCallback
    HRESULT STDMETHODCALLTYPE VideoInputFormatChanged(
        BMDVideoInputFormatChangedEvents,
        IDeckLinkDisplayMode*,
        BMDDetectedVideoInputFormatFlags) override {
        return S_OK;
    }

    HRESULT STDMETHODCALLTYPE VideoInputFrameArrived(
        IDeckLinkVideoInputFrame* videoFrame,
        IDeckLinkAudioInputPacket*) override {
        if (!videoFrame) return S_OK;

        if (videoFrame->GetFlags() & bmdFrameHasNoInputSource) {
            return S_OK;
        }

        void* frameBytes = nullptr;
        IDeckLinkVideoBuffer* buffer = nullptr;
        HRESULT hr = videoFrame->QueryInterface(IID_IDeckLinkVideoBuffer, (void**)&buffer);
        if (FAILED(hr) || !buffer) return S_OK;

        hr = buffer->StartAccess(bmdBufferAccessRead);
        if (FAILED(hr)) {
            buffer->Release();
            return S_OK;
        }

        hr = buffer->GetBytes(&frameBytes);
        if (SUCCEEDED(hr) && frameBytes) {
            long rowBytes = videoFrame->GetRowBytes();
            long height = videoFrame->GetHeight();
            size_t frameSize = static_cast<size_t>(rowBytes) * static_cast<size_t>(height);
            owner_->on_frame(frameBytes, frameSize);
        }

        buffer->EndAccess(bmdBufferAccessRead);
        buffer->Release();
        return S_OK;
    }

private:
    DeckLinkInputCapture* owner_;
#ifdef _WIN32
    LONG ref_count_;
#else
    ULONG ref_count_;
#endif
};

// --- DeckLinkSystem ---

DeckLinkSystem::DeckLinkSystem() {
#ifdef _WIN32
    CoInitializeEx(NULL, COINIT_MULTITHREADED);
#endif
    IDeckLinkIterator* iter = CreateDeckLinkIteratorInstance();
    api_present_ = (iter != nullptr);
    if (iter) iter->Release();
}

DeckLinkSystem::~DeckLinkSystem() {
#ifdef _WIN32
    CoUninitialize();
#endif
}

bool DeckLinkSystem::is_api_present() const {
    return api_present_;
}

rust::Vec<BridgeDeviceInfo> DeckLinkSystem::enumerate() const {
    rust::Vec<BridgeDeviceInfo> result;

    IDeckLinkIterator* iterator = CreateDeckLinkIteratorInstance();
    if (!iterator) return result;

    IDeckLink* deckLink = nullptr;
    uint32_t index = 0;

    while (iterator->Next(&deckLink) == S_OK) {
        BridgeDeviceInfo info;
        info.index = index;

#ifdef _WIN32
        BSTR displayName = nullptr;
        if (deckLink->GetDisplayName(&displayName) == S_OK) {
            info.name = decklink_string_to_rust(displayName);
            SysFreeString(displayName);
        } else {
            info.name = rust::String("Unknown");
        }

        BSTR modelName = nullptr;
        if (deckLink->GetModelName(&modelName) == S_OK) {
            info.model_name = decklink_string_to_rust(modelName);
            SysFreeString(modelName);
        } else {
            info.model_name = rust::String("Unknown");
        }
#else
        const char* displayName = nullptr;
        if (deckLink->GetDisplayName(&displayName) == S_OK && displayName) {
            info.name = decklink_string_to_rust(displayName);
        } else {
            info.name = rust::String("Unknown");
        }

        const char* modelName = nullptr;
        if (deckLink->GetModelName(&modelName) == S_OK && modelName) {
            info.model_name = decklink_string_to_rust(modelName);
        } else {
            info.model_name = rust::String("Unknown");
        }
#endif

        // Check for input capability
        IDeckLinkInput* input = nullptr;
        info.has_input = (deckLink->QueryInterface(IID_IDeckLinkInput, (void**)&input) == S_OK);
        if (input) input->Release();

        // Check for output capability
        IDeckLinkOutput* output = nullptr;
        info.has_output = (deckLink->QueryInterface(IID_IDeckLinkOutput, (void**)&output) == S_OK);
        if (output) output->Release();

        result.push_back(info);
        deckLink->Release();
        index++;
    }

    iterator->Release();
    return result;
}

// --- DeckLinkInputCapture ---

DeckLinkInputCapture::DeckLinkInputCapture(IDeckLinkInput* input, uint32_t width, uint32_t height)
    : input_(input), callback_(nullptr), width_(width), height_(height),
      running_(false), frame_ready_(false) {}

DeckLinkInputCapture::~DeckLinkInputCapture() {
    stop();
    if (input_) {
        input_->Release();
        input_ = nullptr;
    }
}

bool DeckLinkInputCapture::start() {
    if (running_ || !input_) return false;

    callback_ = new InputCallbackImpl(this);
    HRESULT hr = input_->SetCallback(callback_);
    if (FAILED(hr)) {
        callback_->Release();
        callback_ = nullptr;
        return false;
    }

    hr = input_->StartStreams();
    if (FAILED(hr)) {
        input_->SetCallback(nullptr);
        callback_->Release();
        callback_ = nullptr;
        return false;
    }

    running_ = true;
    return true;
}

void DeckLinkInputCapture::stop() {
    if (!running_) return;
    running_ = false;

    if (input_) {
        input_->StopStreams();
        input_->SetCallback(nullptr);
    }

    // Wake up any waiting get_frame calls
    {
        std::lock_guard<std::mutex> lock(mutex_);
        frame_ready_ = true;
        cv_.notify_all();
    }

    if (callback_) {
        callback_->Release();
        callback_ = nullptr;
    }
}

rust::Vec<uint8_t> DeckLinkInputCapture::get_frame(uint32_t timeout_ms) {
    rust::Vec<uint8_t> result;

    std::unique_lock<std::mutex> lock(mutex_);
    if (!frame_ready_) {
        cv_.wait_for(lock, std::chrono::milliseconds(timeout_ms),
                     [this] { return frame_ready_; });
    }

    if (frame_ready_ && !buffer_.empty()) {
        result.reserve(buffer_.size());
        for (uint8_t b : buffer_) {
            result.push_back(b);
        }
        frame_ready_ = false;
    }

    return result;
}

uint32_t DeckLinkInputCapture::frame_width() const {
    return width_;
}

uint32_t DeckLinkInputCapture::frame_height() const {
    return height_;
}

void DeckLinkInputCapture::on_frame(const void* data, size_t size) {
    std::lock_guard<std::mutex> lock(mutex_);
    buffer_.resize(size);
    std::memcpy(buffer_.data(), data, size);
    frame_ready_ = true;
    cv_.notify_one();
}

// --- DeckLinkOutputPlayer ---

DeckLinkOutputPlayer::DeckLinkOutputPlayer(
    IDeckLinkOutput* output, uint32_t mode, uint32_t pixel_format,
    int32_t width, int32_t height, int32_t row_bytes)
    : output_(output), mode_(mode), pixel_format_(pixel_format),
      width_(width), height_(height), row_bytes_(row_bytes),
      running_(false), current_frame_(0) {
    for (int i = 0; i < FRAME_POOL_SIZE; i++) {
        frame_pool_[i] = nullptr;
    }
}

DeckLinkOutputPlayer::~DeckLinkOutputPlayer() {
    stop();
    for (int i = 0; i < FRAME_POOL_SIZE; i++) {
        if (frame_pool_[i]) {
            frame_pool_[i]->Release();
            frame_pool_[i] = nullptr;
        }
    }
    if (output_) {
        output_->Release();
        output_ = nullptr;
    }
}

bool DeckLinkOutputPlayer::start() {
    if (running_ || !output_) return false;

    HRESULT hr = output_->EnableVideoOutput(
        static_cast<BMDDisplayMode>(mode_), bmdVideoOutputFlagDefault);
    if (FAILED(hr)) return false;

    // Pre-allocate frame pool
    for (int i = 0; i < FRAME_POOL_SIZE; i++) {
        hr = output_->CreateVideoFrame(
            width_, height_, row_bytes_,
            static_cast<BMDPixelFormat>(pixel_format_),
            bmdFrameFlagDefault, &frame_pool_[i]);
        if (FAILED(hr)) {
            // Clean up on failure
            for (int j = 0; j < i; j++) {
                frame_pool_[j]->Release();
                frame_pool_[j] = nullptr;
            }
            output_->DisableVideoOutput();
            return false;
        }
    }

    running_ = true;
    return true;
}

void DeckLinkOutputPlayer::stop() {
    if (!running_) return;
    running_ = false;

    if (output_) {
        output_->DisableVideoOutput();
    }
}

bool DeckLinkOutputPlayer::send_frame(rust::Slice<const uint8_t> data) {
    if (!running_ || !output_) return false;

    IDeckLinkMutableVideoFrame* frame = frame_pool_[current_frame_];
    if (!frame) return false;

    // Copy data into the frame buffer
    IDeckLinkVideoBuffer* buffer = nullptr;
    HRESULT hr = frame->QueryInterface(IID_IDeckLinkVideoBuffer, (void**)&buffer);
    if (FAILED(hr) || !buffer) return false;

    hr = buffer->StartAccess(bmdBufferAccessWrite);
    if (FAILED(hr)) {
        buffer->Release();
        return false;
    }

    void* frameBytes = nullptr;
    hr = buffer->GetBytes(&frameBytes);
    if (SUCCEEDED(hr) && frameBytes) {
        size_t copy_size = static_cast<size_t>(row_bytes_) * static_cast<size_t>(height_);
        if (data.size() >= copy_size) {
            std::memcpy(frameBytes, data.data(), copy_size);
        }
    }

    buffer->EndAccess(bmdBufferAccessWrite);
    buffer->Release();

    hr = output_->DisplayVideoFrameSync(frame);
    current_frame_ = (current_frame_ + 1) % FRAME_POOL_SIZE;

    return SUCCEEDED(hr);
}

// --- Factory functions ---

std::unique_ptr<DeckLinkSystem> create_system() {
    return std::make_unique<DeckLinkSystem>();
}

std::unique_ptr<DeckLinkInputCapture> create_capture(
    const DeckLinkSystem& sys, uint32_t device_idx, uint32_t mode, uint32_t fmt) {
    if (!sys.is_api_present()) return nullptr;

    IDeckLinkIterator* iterator = CreateDeckLinkIteratorInstance();
    if (!iterator) return nullptr;

    IDeckLink* deckLink = nullptr;
    uint32_t idx = 0;
    while (iterator->Next(&deckLink) == S_OK) {
        if (idx == device_idx) break;
        deckLink->Release();
        deckLink = nullptr;
        idx++;
    }
    iterator->Release();

    if (!deckLink) return nullptr;

    IDeckLinkInput* input = nullptr;
    HRESULT hr = deckLink->QueryInterface(IID_IDeckLinkInput, (void**)&input);
    deckLink->Release();

    if (FAILED(hr) || !input) return nullptr;

    // Enable video input
    hr = input->EnableVideoInput(
        static_cast<BMDDisplayMode>(mode),
        static_cast<BMDPixelFormat>(fmt),
        bmdVideoInputFlagDefault);
    if (FAILED(hr)) {
        input->Release();
        return nullptr;
    }

    // Get resolution from display mode
    IDeckLinkDisplayMode* displayMode = nullptr;
    uint32_t width = 1920, height = 1080;
    hr = input->GetDisplayMode(static_cast<BMDDisplayMode>(mode), &displayMode);
    if (SUCCEEDED(hr) && displayMode) {
        width = static_cast<uint32_t>(displayMode->GetWidth());
        height = static_cast<uint32_t>(displayMode->GetHeight());
        displayMode->Release();
    }

    return std::make_unique<DeckLinkInputCapture>(input, width, height);
}

std::unique_ptr<DeckLinkOutputPlayer> create_player(
    const DeckLinkSystem& sys, uint32_t device_idx, uint32_t mode, uint32_t fmt) {
    if (!sys.is_api_present()) return nullptr;

    IDeckLinkIterator* iterator = CreateDeckLinkIteratorInstance();
    if (!iterator) return nullptr;

    IDeckLink* deckLink = nullptr;
    uint32_t idx = 0;
    while (iterator->Next(&deckLink) == S_OK) {
        if (idx == device_idx) break;
        deckLink->Release();
        deckLink = nullptr;
        idx++;
    }
    iterator->Release();

    if (!deckLink) return nullptr;

    IDeckLinkOutput* output = nullptr;
    HRESULT hr = deckLink->QueryInterface(IID_IDeckLinkOutput, (void**)&output);
    deckLink->Release();

    if (FAILED(hr) || !output) return nullptr;

    // Get resolution and row bytes from display mode
    IDeckLinkDisplayMode* displayMode = nullptr;
    int32_t width = 1920, height = 1080;
    hr = output->GetDisplayMode(static_cast<BMDDisplayMode>(mode), &displayMode);
    if (SUCCEEDED(hr) && displayMode) {
        width = static_cast<int32_t>(displayMode->GetWidth());
        height = static_cast<int32_t>(displayMode->GetHeight());
        displayMode->Release();
    }

    int32_t row_bytes = 0;
    hr = output->RowBytesForPixelFormat(static_cast<BMDPixelFormat>(fmt), width, &row_bytes);
    if (FAILED(hr)) {
        // Fallback calculation
        switch (fmt) {
            case 0x32767579: // UYVY
                row_bytes = width * 2;
                break;
            case 0x42475241: // BGRA
                row_bytes = width * 4;
                break;
            default:
                row_bytes = width * 2;
                break;
        }
    }

    return std::make_unique<DeckLinkOutputPlayer>(
        output, mode, fmt, width, height, row_bytes);
}

} // namespace momo
