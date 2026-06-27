#version 330
in vec2 fragTexCoord;
in vec4 fragColor;
out vec4 finalColor;
uniform sampler2D texture0;
uniform float time;

float hash(vec2 p) {
    p = fract(p * vec2(123.34, 456.21));
    p += dot(p, p + 45.32);
    return fract(p.x * p.y);
}

float star(vec2 uv, vec2 center, float size) {
    float d = length(uv - center);
    return smoothstep(size, 0.0, d);
}

vec3 stars(vec2 uv) {
    vec3 col = vec3(0.0);
    vec2 grid = floor(uv * 80.0);
    vec2 cell = fract(uv * 80.0);

    for (int x = -1; x <= 1; x++) {
        for (int y = -1; y <= 1; y++) {
            vec2 g = grid + vec2(x, y);
            float h = hash(g);
            if (h > 0.99) {
                vec2 offset = vec2(hash(g + 0.1), hash(g + 0.2));
                vec2 starPos = vec2(x, y) + offset - cell;
                float size = 0.018 + hash(g + 0.3) * 0.48;
                float brightness = star(vec2(0.0), starPos, size);

                // Shimmer: each star twinkles at its own frequency and phase
                float freq = 1.5 + hash(g + 0.5) * 0.0005;
                float phase = hash(g + 0.6) * 6.2832;
                float shimmer = 0.1 + 0.6 * pow(abs(sin(time * freq + phase)), 1.5);

                // Subtle violet-white color variation
                vec3 starColor = mix(vec3(0.85, 0.82, 1.0), vec3(1.0, 0.98, 1.0), hash(g + 0.7));
                col += brightness * shimmer * starColor * (0.6 + hash(g + 0.4) * 0.6);
            }
        }
    }
    return col;
}

void main()
{
    vec2 uv = fragTexCoord;

    // Curvature
    vec2 dc = abs(0.5 - uv);
    dc *= dc;
    uv.x -= 0.5; uv.x *= 1.0 + (dc.y * 0.3); uv.x += 0.5;
    uv.y -= 0.5; uv.y *= 1.0 + (dc.x * 0.4); uv.y += 0.5;

    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
        finalColor = vec4(0.0, 0.0, 0.0, 1.0);
    } else {
        vec4 texel = texture(texture0, uv);

        // Scanlines
        texel.rgb *= 0.9 + 0.1 * sin(uv.y * 720.0 * 3.14159);

        // Vignette
        float vig = uv.x * uv.y * (1.0 - uv.x) * (1.0 - uv.y);
        vig = pow(vig * 16.0, 0.25);
        texel.rgb *= vig;

        // Blue -> Violet shift
        float blueness = max(texel.b - max(texel.r, texel.g), 0.0);
        float pulse = 0.5 + 0.5 * sin(time * 0.1);
        texel.r += blueness * pulse * 0.6;

        // Stars in top third
    if (uv.y > 0.90) {
        float fade = smoothstep(0.667, 0.78, uv.y);
        vec3 starGlow = stars(uv) * fade;
        starGlow *= 0.9 + 0.1 * sin(uv.y * 720.0 * 3.14159);
        texel.rgb += starGlow;
    }
        finalColor = texel * fragColor;
    }
}