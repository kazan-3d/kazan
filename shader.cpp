#ifndef RECURSIVE_INVOCATION
#define UNIQUEIFY3(v, c) v ## _ ## c
#define UNIQUEIFY2(v, c) UNIQUEIFY3(v, c)
#define UNIQUEIFY(v) UNIQUEIFY2(v, __COUNTER__)
// following is code to include the body of this file pow(2, 7) times
#define RECURSIVE_INVOCATION 1
#include "shader.cpp"
#undef RECURSIVE_INVOCATION
#define RECURSIVE_INVOCATION 1
#include "shader.cpp"
#elif RECURSIVE_INVOCATION == 1
#undef RECURSIVE_INVOCATION
#define RECURSIVE_INVOCATION 2
#include "shader.cpp"
#undef RECURSIVE_INVOCATION
#define RECURSIVE_INVOCATION 2
#include "shader.cpp"
#elif RECURSIVE_INVOCATION == 2
#undef RECURSIVE_INVOCATION
#define RECURSIVE_INVOCATION 3
#include "shader.cpp"
#undef RECURSIVE_INVOCATION
#define RECURSIVE_INVOCATION 3
#include "shader.cpp"
#elif RECURSIVE_INVOCATION == 3
#undef RECURSIVE_INVOCATION
#define RECURSIVE_INVOCATION 4
#include "shader.cpp"
#undef RECURSIVE_INVOCATION
#define RECURSIVE_INVOCATION 4
#include "shader.cpp"
#elif RECURSIVE_INVOCATION == 4
#undef RECURSIVE_INVOCATION
#define RECURSIVE_INVOCATION 5
#include "shader.cpp"
#undef RECURSIVE_INVOCATION
#define RECURSIVE_INVOCATION 5
#include "shader.cpp"
#elif RECURSIVE_INVOCATION == 5
#undef RECURSIVE_INVOCATION
#define RECURSIVE_INVOCATION 6
#include "shader.cpp"
#undef RECURSIVE_INVOCATION
#define RECURSIVE_INVOCATION 6
#include "shader.cpp"
#elif RECURSIVE_INVOCATION == 6
#undef RECURSIVE_INVOCATION
#define RECURSIVE_INVOCATION 7
#include "shader.cpp"
#undef RECURSIVE_INVOCATION
#define RECURSIVE_INVOCATION 7
#include "shader.cpp"
#elif RECURSIVE_INVOCATION == 7
#include <cstdint>
#include <cmath>
#include <limits>

namespace UNIQUEIFY(shader)
{
constexpr float max(float a, float b) noexcept
{
    return a > b ? a : b;
}

struct vec2
{
    float x;
    float y;
    vec2() = default;
    constexpr vec2(float v) noexcept : x(v), y(v)
    {
    }
    constexpr vec2(float x, float y) noexcept : x(x), y(y)
    {
    }
    friend constexpr vec2 operator *(vec2 a, float b) noexcept
    {
        return vec2(a.x * b, a.y * b);
    }
    friend constexpr vec2 operator *(float a, vec2 b) noexcept
    {
        return vec2(a * b.x, a * b.y);
    }
    friend constexpr vec2 operator +(vec2 a, vec2 b) noexcept
    {
        return vec2(a.x + b.x, a.y + b.y);
    }
    friend constexpr vec2 operator -(vec2 a, vec2 b) noexcept
    {
        return vec2(a.x - b.x, a.y - b.y);
    }
    friend constexpr vec2 operator /(vec2 a, vec2 b) noexcept
    {
        return vec2(a.x / b.x, a.y / b.y);
    }
};

struct vec3
{
    float x;
    float y;
    float z;
    vec3() = default;
    constexpr vec3(float v) noexcept : x(v), y(v), z(v)
    {
    }
    constexpr vec3(float x, float y, float z) noexcept : x(x), y(y), z(z)
    {
    }
    constexpr vec3(vec2 xy, float z) noexcept : x(xy.x), y(xy.y), z(z)
    {
    }
    constexpr vec3 xzy() const noexcept
    {
        return vec3(x, z, y);
    }
    constexpr vec3 xyz() const noexcept
    {
        return vec3(x, y, z);
    }
    constexpr vec3 rgb() const noexcept
    {
        return vec3(x, y, z);
    }
    constexpr float r() const noexcept
    {
        return x;
    }
    constexpr float g() const noexcept
    {
        return y;
    }
    constexpr float b() const noexcept
    {
        return z;
    }
    friend constexpr vec3 operator /(vec3 a, vec3 b) noexcept
    {
        return vec3(a.x / b.x, a.y / b.y, a.z / b.z);
    }
    friend constexpr vec3 operator *(vec3 a, vec3 b) noexcept
    {
        return vec3(a.x * b.x, a.y * b.y, a.z * b.z);
    }
    friend constexpr vec3 operator +(vec3 a, vec3 b) noexcept
    {
        return vec3(a.x + b.x, a.y + b.y, a.z + b.z);
    }
    friend constexpr vec3 operator -(vec3 a, vec3 b) noexcept
    {
        return vec3(a.x - b.x, a.y - b.y, a.z - b.z);
    }
};

struct vec4
{
    float x;
    float y;
    float z;
    float w;
    vec4() = default;
    constexpr vec4(float v) noexcept : x(v), y(v), z(v), w(v)
    {
    }
    constexpr vec4(float x, float y, float z, float w) noexcept : x(x), y(y), z(z), w(w)
    {
    }
    constexpr vec4(vec2 xy, float z, float w) noexcept : x(xy.x), y(xy.y), z(z), w(w)
    {
    }
    constexpr vec4(vec3 xyz, float w) noexcept : x(xyz.x), y(xyz.y), z(xyz.z), w(w)
    {
    }
    constexpr vec3 xyz() const noexcept
    {
        return {x, y, z};
    }
    constexpr vec2 xy() const noexcept
    {
        return {x, y};
    }
    friend constexpr vec4 operator *(vec4 a, float b) noexcept
    {
        return vec4(a.x * b, a.y * b, a.z * b, a.w * b);
    }
    friend constexpr vec4 operator *(float a, vec4 b) noexcept
    {
        return vec4(a * b.x, a * b.y, a * b.z, a * b.w);
    }
    friend constexpr vec4 operator /(vec4 a, float b) noexcept
    {
        return vec4(a.x / b, a.y / b, a.z / b, a.w / b);
    }
    constexpr vec4 &operator /=(float v) noexcept
    {
        return *this = *this / v;
    }
    friend constexpr vec4 operator +(vec4 a, vec4 b) noexcept
    {
        return vec4(a.x + b.x, a.y + b.y, a.z + b.z, a.w + b.w);
    }
    friend constexpr vec4 operator -(vec4 a, vec4 b) noexcept
    {
        return vec4(a.x - b.x, a.y - b.y, a.z - b.z, a.w - b.w);
    }
    constexpr vec4 &operator +=(vec4 v) noexcept
    {
        return *this = *this + v;
    }
};

constexpr float dot(vec3 a, vec3 b) noexcept
{
    return a.x * b.x + a.y * b.y + a.z * b.z;
}

inline float length(vec3 v) noexcept
{
    return std::sqrt(dot(v, v));
}

inline float distance(vec3 a, vec3 b) noexcept
{
    return length(a - b);
}

inline vec3 normalize(vec3 v) noexcept
{
    return v / length(v);
}

struct mat4
{
    float values[4][4] = {{1, 0, 0, 0}, {0, 1, 0, 0}, {0, 0, 1, 0}, {0, 0, 0, 1}};
    constexpr mat4() noexcept {}
    constexpr mat4(float value_0_0, float value_0_1, float value_0_2, float value_0_3,
                   float value_1_0, float value_1_1, float value_1_2, float value_1_3,
                   float value_2_0, float value_2_1, float value_2_2, float value_2_3,
                   float value_3_0, float value_3_1, float value_3_2, float value_3_3) noexcept
        : values{
            {value_0_0, value_0_1, value_0_2, value_0_3},
            {value_1_0, value_1_1, value_1_2, value_1_3},
            {value_2_0, value_2_1, value_2_2, value_2_3},
            {value_3_0, value_3_1, value_3_2, value_3_3},
        }
    {
    }
    friend constexpr mat4 operator *(float a, const mat4 &b) noexcept
    {
        return mat4(a * b.values[0][0], a * b.values[0][1], a * b.values[0][2], a * b.values[0][3],
                    a * b.values[1][0], a * b.values[1][1], a * b.values[1][2], a * b.values[1][3],
                    a * b.values[2][0], a * b.values[2][1], a * b.values[2][2], a * b.values[2][3],
                    a * b.values[3][0], a * b.values[3][1], a * b.values[3][2], a * b.values[3][3]);
    }
    friend constexpr mat4 operator *(const mat4 &a, float b) noexcept
    {
        return mat4(a.values[0][0] * b, a.values[0][1] * b, a.values[0][2] * b, a.values[0][3] * b,
                    a.values[1][0] * b, a.values[1][1] * b, a.values[1][2] * b, a.values[1][3] * b,
                    a.values[2][0] * b, a.values[2][1] * b, a.values[2][2] * b, a.values[2][3] * b,
                    a.values[3][0] * b, a.values[3][1] * b, a.values[3][2] * b, a.values[3][3] * b);
    }
    friend constexpr vec4 operator *(const mat4 &m, vec4 v) noexcept
    {
        return vec4(m.values[0][2]*v.z+m.values[0][1]*v.y+m.values[0][0]*v.x
                         +m.values[0][3]*v.w,
 m.values[1][2]*v.z+m.values[1][1]*v.y+m.values[1][0]*v.x
                         +m.values[1][3]*v.w,
 m.values[2][2]*v.z+m.values[2][1]*v.y+m.values[2][0]*v.x
                         +m.values[2][3]*v.w,
 m.values[3][2]*v.z+m.values[3][1]*v.y+m.values[3][0]*v.x
                         +m.values[3][3]*v.w);
    }
};

constexpr float determinant(const mat4 &m) noexcept
{
    return ((m.values[0][1]*m.values[1][2]
 -m.values[0][2]*m.values[1][1])
 *m.values[2][0]
 +(m.values[0][2]*m.values[1][0]
  -m.values[0][0]*m.values[1][2])
  *m.values[2][1]
 +(m.values[0][0]*m.values[1][1]
  -m.values[0][1]*m.values[1][0])
  *m.values[2][2])
 *m.values[3][3]
 +((m.values[0][3]*m.values[1][1]
  -m.values[0][1]*m.values[1][3])
  *m.values[2][0]
  +(m.values[0][0]*m.values[1][3]
   -m.values[0][3]*m.values[1][0])
   *m.values[2][1]
  +(m.values[0][1]*m.values[1][0]
   -m.values[0][0]*m.values[1][1])
   *m.values[2][3])
  *m.values[3][2]
 +((m.values[0][2]*m.values[1][3]
  -m.values[0][3]*m.values[1][2])
  *m.values[2][0]
  +(m.values[0][3]*m.values[1][0]
   -m.values[0][0]*m.values[1][3])
   *m.values[2][2]
  +(m.values[0][0]*m.values[1][2]
   -m.values[0][2]*m.values[1][0])
   *m.values[2][3])
  *m.values[3][1]
 +((m.values[0][3]*m.values[1][2]
  -m.values[0][2]*m.values[1][3])
  *m.values[2][1]
  +(m.values[0][1]*m.values[1][3]
   -m.values[0][3]*m.values[1][1])
   *m.values[2][2]
  +(m.values[0][2]*m.values[1][1]
   -m.values[0][1]*m.values[1][2])
   *m.values[2][3])
  *m.values[3][0];
}

constexpr mat4 inverse(const mat4 &m) noexcept
{
    return 1.0f / determinant(m) * mat4((m.values[1][1]*m.values[2][2]
         -m.values[1][2]*m.values[2][1])
         *m.values[3][3]
         +(m.values[1][3]*m.values[2][1]
          -m.values[1][1]*m.values[2][3])
          *m.values[3][2]
         +(m.values[1][2]*m.values[2][3]
          -m.values[1][3]*m.values[2][2])
          *m.values[3][1],
        (m.values[0][2]*m.values[2][1]
         -m.values[0][1]*m.values[2][2])
         *m.values[3][3]
         +(m.values[0][1]*m.values[2][3]
          -m.values[0][3]*m.values[2][1])
          *m.values[3][2]
         +(m.values[0][3]*m.values[2][2]
          -m.values[0][2]*m.values[2][3])
          *m.values[3][1],
        (m.values[0][1]*m.values[1][2]
         -m.values[0][2]*m.values[1][1])
         *m.values[3][3]
         +(m.values[0][3]*m.values[1][1]
          -m.values[0][1]*m.values[1][3])
          *m.values[3][2]
         +(m.values[0][2]*m.values[1][3]
          -m.values[0][3]*m.values[1][2])
          *m.values[3][1],
        (m.values[0][2]*m.values[1][1]
         -m.values[0][1]*m.values[1][2])
         *m.values[2][3]
         +(m.values[0][1]*m.values[1][3]
          -m.values[0][3]*m.values[1][1])
          *m.values[2][2]
         +(m.values[0][3]*m.values[1][2]
          -m.values[0][2]*m.values[1][3])
          *m.values[2][1],
        (m.values[1][2]*m.values[2][0]
         -m.values[1][0]*m.values[2][2])
         *m.values[3][3]
         +(m.values[1][0]*m.values[2][3]
          -m.values[1][3]*m.values[2][0])
          *m.values[3][2]
         +(m.values[1][3]*m.values[2][2]
          -m.values[1][2]*m.values[2][3])
          *m.values[3][0],
        (m.values[0][0]*m.values[2][2]
         -m.values[0][2]*m.values[2][0])
         *m.values[3][3]
         +(m.values[0][3]*m.values[2][0]
          -m.values[0][0]*m.values[2][3])
          *m.values[3][2]
         +(m.values[0][2]*m.values[2][3]
          -m.values[0][3]*m.values[2][2])
          *m.values[3][0],
        (m.values[0][2]*m.values[1][0]
         -m.values[0][0]*m.values[1][2])
         *m.values[3][3]
         +(m.values[0][0]*m.values[1][3]
          -m.values[0][3]*m.values[1][0])
          *m.values[3][2]
         +(m.values[0][3]*m.values[1][2]
          -m.values[0][2]*m.values[1][3])
          *m.values[3][0],
        (m.values[0][0]*m.values[1][2]
         -m.values[0][2]*m.values[1][0])
         *m.values[2][3]
         +(m.values[0][3]*m.values[1][0]
          -m.values[0][0]*m.values[1][3])
          *m.values[2][2]
         +(m.values[0][2]*m.values[1][3]
          -m.values[0][3]*m.values[1][2])
          *m.values[2][0],
        (m.values[1][0]*m.values[2][1]
         -m.values[1][1]*m.values[2][0])
         *m.values[3][3]
         +(m.values[1][3]*m.values[2][0]
          -m.values[1][0]*m.values[2][3])
          *m.values[3][1]
         +(m.values[1][1]*m.values[2][3]
          -m.values[1][3]*m.values[2][1])
          *m.values[3][0],
        (m.values[0][1]*m.values[2][0]
         -m.values[0][0]*m.values[2][1])
         *m.values[3][3]
         +(m.values[0][0]*m.values[2][3]
          -m.values[0][3]*m.values[2][0])
          *m.values[3][1]
         +(m.values[0][3]*m.values[2][1]
          -m.values[0][1]*m.values[2][3])
          *m.values[3][0],
        (m.values[0][0]*m.values[1][1]
         -m.values[0][1]*m.values[1][0])
         *m.values[3][3]
         +(m.values[0][3]*m.values[1][0]
          -m.values[0][0]*m.values[1][3])
          *m.values[3][1]
         +(m.values[0][1]*m.values[1][3]
          -m.values[0][3]*m.values[1][1])
          *m.values[3][0],
        (m.values[0][1]*m.values[1][0]
         -m.values[0][0]*m.values[1][1])
         *m.values[2][3]
         +(m.values[0][0]*m.values[1][3]
          -m.values[0][3]*m.values[1][0])
          *m.values[2][1]
         +(m.values[0][3]*m.values[1][1]
          -m.values[0][1]*m.values[1][3])
          *m.values[2][0],
        (m.values[1][1]*m.values[2][0]
         -m.values[1][0]*m.values[2][1])
         *m.values[3][2]
         +(m.values[1][0]*m.values[2][2]
          -m.values[1][2]*m.values[2][0])
          *m.values[3][1]
         +(m.values[1][2]*m.values[2][1]
          -m.values[1][1]*m.values[2][2])
          *m.values[3][0],
        (m.values[0][0]*m.values[2][1]
         -m.values[0][1]*m.values[2][0])
         *m.values[3][2]
         +(m.values[0][2]*m.values[2][0]
          -m.values[0][0]*m.values[2][2])
          *m.values[3][1]
         +(m.values[0][1]*m.values[2][2]
          -m.values[0][2]*m.values[2][1])
          *m.values[3][0],
        (m.values[0][1]*m.values[1][0]
         -m.values[0][0]*m.values[1][1])
         *m.values[3][2]
         +(m.values[0][0]*m.values[1][2]
          -m.values[0][2]*m.values[1][0])
          *m.values[3][1]
         +(m.values[0][2]*m.values[1][1]
          -m.values[0][1]*m.values[1][2])
          *m.values[3][0],
        (m.values[0][0]*m.values[1][1]
         -m.values[0][1]*m.values[1][0])
         *m.values[2][2]
         +(m.values[0][2]*m.values[1][0]
          -m.values[0][0]*m.values[1][2])
          *m.values[2][1]
         +(m.values[0][1]*m.values[1][2]
          -m.values[0][2]*m.values[1][1])
          *m.values[2][0]);
}

struct pixel
{
    std::uint8_t r, g, b, a;
    constexpr operator vec4() const noexcept
    {
        constexpr float scale_factor = 1.0 / std::numeric_limits<std::uint8_t>::max();
        return vec4(r, g, b, a) * scale_factor;
    }
};

struct sampler2D
{
    const pixel *pixels;
    std::size_t width;
    std::size_t height;
    vec4 get_pixel_int(int x, int y) const noexcept
    {
        if(x < 0)
            x = 0;
        else if(static_cast<std::size_t>(x) > width - 1)
            x = width - 1;
        if(y < 0)
            y = 0;
        else if(static_cast<std::size_t>(y) > height - 1)
            y = height - 1;
        return pixels[static_cast<std::size_t>(x) + width * static_cast<std::size_t>(y)];
    }
    vec4 get_pixel(vec2 position) const noexcept
    {
        // bilinear interpolation
        int min_x = position.x; // works if position.x >= 0
        int max_x = min_x + 1;
        position.x -= min_x;
        int min_y = position.y; // works if position.y >= 0
        int max_y = min_y + 1;
        position.y -= min_y;
        vec4 min_min_value = get_pixel_int(min_x, min_y);
        vec4 max_min_value = get_pixel_int(max_x, min_y);
        vec4 min_max_value = get_pixel_int(min_x, max_y);
        vec4 max_max_value = get_pixel_int(max_x, max_y);
        vec4 min_interp_value = min_min_value + position.y * (min_max_value - min_min_value);
        vec4 max_interp_value = max_min_value + position.y * (max_max_value - max_min_value);
        return min_interp_value + position.x * (max_interp_value - min_interp_value);
    }
};

vec4 texture(const sampler2D &sampler, vec2 uv) noexcept
{
    return sampler.get_pixel(uv);
}

extern vec4 gl_FragCoord;

// shader translated from SuperTuxKart data/shaders/rh.frag
// https://github.com/supertuxkart/stk-code/blob/20ea7ca2711f0cbe5320b4877a5d332b3b935893/data/shaders/rh.frag

// From http://graphics.cs.aueb.gr/graphics/research_illumination.html
// "Real-Time Diffuse Global Illumination Using Radiance Hints"
// paper and shader code

float R_wcs = 10.f;            // Rmax: maximum sampling distance (in WCS units)
vec3 extents;
mat4 RHMatrix;
mat4 RSMMatrix;
extern sampler2D dtex;
extern sampler2D ctex;
extern sampler2D ntex;
vec3 suncol;

int slice;
vec4 SHRed;
vec4 SHGreen;
vec4 SHBlue;

vec3 resolution = vec3(32, 16, 32);
#define SAMPLES 16

static vec4 SHBasis (const vec3 dir) noexcept
{
    float   L00  = 0.282095f;
    float   L1_1 = 0.488603f * dir.y;
    float   L10  = 0.488603f * dir.z;
    float   L11  = 0.488603f * dir.x;
    return vec4 (L11, L1_1, L10, L00);
}

static vec4 DirToSh(vec3 dir, float flux) noexcept
{
    return SHBasis (dir) * flux;
}

// We need to manually unroll the loop, otherwise Nvidia driver crashes.
static void loop(int i,
          vec3 RHcenter,vec3 RHCellSize, vec2 RHuv, float RHdepth,
          vec4 &SHr, vec4 &SHg, vec4 &SHb) noexcept
{
    // produce a new sample location on the RSM texture
    float alpha = (i + .5f) / SAMPLES;
    float theta = 2.f * 3.14f * 7.f * alpha;
    float h = alpha;
    vec2 offset = h * vec2(cos(theta), sin(theta));
    vec2 uv = RHuv + offset * 0.01f;

    // Get world position and normal from the RSM sample
    float depth = texture(dtex, uv).x;
    vec4 RSMPos = inverse(RSMMatrix) * (2.f * vec4(uv, depth, 1.f) - 1.f);
    RSMPos /= RSMPos.w;
    vec3 RSMAlbedo = texture(ctex, uv).xyz();
    vec3 normal = normalize(2.f * texture(ntex, uv).xyz() - 1.f);

    // Sampled location inside the RH cell
    vec3 offset3d = vec3(uv, 0);
    vec3 SamplePos = RHcenter + .5f * offset3d.xzy() * RHCellSize;

    // Normalize distance to RSM sample
    float dist = distance(SamplePos, RSMPos.xyz()) / R_wcs;
    // Determine the incident direction.
    // Avoid very close samples (and numerical instability problems)
    vec3 RSM_to_RH_dir = (dist <= 0.1f) ? vec3(0.) : normalize(SamplePos - RSMPos.xyz());
    float dotprod = max(dot(RSM_to_RH_dir, normal.xyz()), 0.f);
    float factor = dotprod / (0.1f + dist * dist);

    vec3 color = RSMAlbedo.rgb() * factor * suncol.rgb();

    SHr += DirToSh(RSM_to_RH_dir, color.r());
    SHg += DirToSh(RSM_to_RH_dir, color.g());
    SHb += DirToSh(RSM_to_RH_dir, color.b());
}

void shader_main(void) noexcept
{
    vec3 normalizedRHCenter = 2.f * vec3(gl_FragCoord.xy(), slice) / resolution - 1.f;
    vec3 RHcenter = (RHMatrix * vec4(normalizedRHCenter * extents, 1.f)).xyz();

    vec4 ShadowProjectedRH = RSMMatrix * vec4(RHcenter, 1.f);

    vec3 RHCellSize = extents / resolution;
    vec2 RHuv = .5f * ShadowProjectedRH.xy() / ShadowProjectedRH.w + .5f;
    float RHdepth = .5f * ShadowProjectedRH.z / ShadowProjectedRH.w + .5f;

    vec4  SHr = vec4(0.f);
    vec4  SHg = vec4(0.f);
    vec4  SHb = vec4(0.f);

    //int x = int(gl_FragCoord.x), y = int(gl_FragCoord.y);
    //float phi = 30.f * (x ^ y) + 10.f * x * y;

    loop(0, RHcenter, RHCellSize, RHuv, RHdepth, SHr, SHg, SHb);
    loop(1, RHcenter, RHCellSize, RHuv, RHdepth, SHr, SHg, SHb);
    loop(2, RHcenter, RHCellSize, RHuv, RHdepth, SHr, SHg, SHb);
    loop(3, RHcenter, RHCellSize, RHuv, RHdepth, SHr, SHg, SHb);
    loop(4, RHcenter, RHCellSize, RHuv, RHdepth, SHr, SHg, SHb);
    loop(5, RHcenter, RHCellSize, RHuv, RHdepth, SHr, SHg, SHb);
    loop(6, RHcenter, RHCellSize, RHuv, RHdepth, SHr, SHg, SHb);
    loop(7, RHcenter, RHCellSize, RHuv, RHdepth, SHr, SHg, SHb);
    loop(8, RHcenter, RHCellSize, RHuv, RHdepth, SHr, SHg, SHb);
    loop(9, RHcenter, RHCellSize, RHuv, RHdepth, SHr, SHg, SHb);
    loop(10, RHcenter, RHCellSize, RHuv, RHdepth, SHr, SHg, SHb);
    loop(11, RHcenter, RHCellSize, RHuv, RHdepth, SHr, SHg, SHb);
    loop(12, RHcenter, RHCellSize, RHuv, RHdepth, SHr, SHg, SHb);
    loop(13, RHcenter, RHCellSize, RHuv, RHdepth, SHr, SHg, SHb);
    loop(14, RHcenter, RHCellSize, RHuv, RHdepth, SHr, SHg, SHb);
    loop(15, RHcenter, RHCellSize, RHuv, RHdepth, SHr, SHg, SHb);

    SHr /= 3.14159f * SAMPLES;
    SHg /= 3.14159f * SAMPLES;
    SHb /= 3.14159f * SAMPLES;

    SHRed = SHr;
    SHGreen = SHg;
    SHBlue = SHb;
}
}
#endif
