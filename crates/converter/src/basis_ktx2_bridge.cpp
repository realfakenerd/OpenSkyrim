#include <cstddef>
#include <cstdint>

// basis-universal-rs 0.3 compiles these stable low-level encoder entry points,
// but does not expose them from its Rust API. Keeping the bridge header-free
// avoids vendoring the codec and leaves ownership with the upstream crate.
namespace basisu {
struct image_stats;
void* basis_compress(const std::uint8_t* rgba, std::uint32_t width,
                     std::uint32_t height, std::uint32_t pitch_in_pixels,
                     std::uint32_t flags_and_quality, float uastc_rdo_quality,
                     std::size_t* size, image_stats* stats);
void basis_free_data(void* data);
}

extern "C" void* opensky_basis_compress_ktx2(
    const std::uint8_t* rgba, std::uint32_t width, std::uint32_t height,
    std::uint32_t flags_and_quality, float uastc_rdo_quality,
    std::size_t* size) {
    return basisu::basis_compress(rgba, width, height, width,
                                  flags_and_quality, uastc_rdo_quality,
                                  size, nullptr);
}

extern "C" void opensky_basis_free(void* data) {
    basisu::basis_free_data(data);
}
