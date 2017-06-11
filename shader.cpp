#include <cstdint>
#include <cmath>

// shader translated from SuperTuxKart data/shaders/rh.frag
// https://github.com/supertuxkart/stk-code/blob/20ea7ca2711f0cbe5320b4877a5d332b3b935893/data/shaders/rh.frag

// From http://graphics.cs.aueb.gr/graphics/research_illumination.html
// "Real-Time Diffuse Global Illumination Using Radiance Hints"
// paper and shader code

struct vec3
{
    float x;
    float y;
    float z;
    vec3() = default;
    constexpr vec3(float v) noexcept : x(v), y(v), z(v)
    {
    }
};

float R_wcs = 10.f;            // Rmax: maximum sampling distance (in WCS units)
vec3 extents;
mat4 RHMatrix;
mat4 RSMMatrix;
sampler2D dtex;
sampler2D ctex;
sampler2D ntex;
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
    vec3 RSMAlbedo = texture(ctex, uv).xyz;
    vec3 normal = normalize(2.f * texture(ntex, uv).xyz - 1.f);

    // Sampled location inside the RH cell
    vec3 offset3d = vec3(uv, 0);
    vec3 SamplePos = RHcenter + .5f * offset3d.xzy * RHCellSize;

    // Normalize distance to RSM sample
    float dist = distance(SamplePos, RSMPos.xyz) / R_wcs;
    // Determine the incident direction.
    // Avoid very close samples (and numerical instability problems)
    vec3 RSM_to_RH_dir = (dist <= 0.1f) ? vec3(0.) : normalize(SamplePos - RSMPos.xyz);
    float dotprod = max(dot(RSM_to_RH_dir, normal.xyz), 0.f);
    float factor = dotprod / (0.1f + dist * dist);

    vec3 color = RSMAlbedo.rgb * factor * suncol.rgb;

    SHr += DirToSh(RSM_to_RH_dir, color.r);
    SHg += DirToSh(RSM_to_RH_dir, color.g);
    SHb += DirToSh(RSM_to_RH_dir, color.b);
}

void shader_main(void) noexcept
{
    vec3 normalizedRHCenter = 2.f * vec3(gl_FragCoord.xy, slice) / resolution - 1.f;
    vec3 RHcenter = (RHMatrix * vec4(normalizedRHCenter * extents, 1.f)).xyz;

    vec4 ShadowProjectedRH = RSMMatrix * vec4(RHcenter, 1.f);

    vec3 RHCellSize = extents / resolution;
    vec2 RHuv = .5f * ShadowProjectedRH.xy / ShadowProjectedRH.w + .5f;
    float RHdepth = .5f * ShadowProjectedRH.z / ShadowProjectedRH.w + .5f;

    vec4  SHr = vec4(0.f);
    vec4  SHg = vec4(0.f);
    vec4  SHb = vec4(0.f);

    int x = int(gl_FragCoord.x), y = int(gl_FragCoord.y);
    float phi = 30.f * (x ^ y) + 10.f * x * y;

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
