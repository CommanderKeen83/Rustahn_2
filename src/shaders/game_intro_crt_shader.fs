#version 330
in vec2 fragTexCoord;
in vec4 fragColor;
out vec4 finalColor;
uniform sampler2D texture0;
uniform float time;

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
        texel.rgb *= 0.9 + 0.1 * sin(uv.y * 600.0 * 3.14159);
        // Vignette
        float vig = uv.x * uv.y * (1.0 - uv.x) * (1.0 - uv.y);
        vig = pow(vig * 16.0, 0.25);
        texel.rgb *= vig;
        // Blue -> Violet shift
        float blueness = max(texel.b - max(texel.r, texel.g), 0.0);
        float pulse = 0.5 + 0.5 * sin(time * 0.1);
        texel.r += blueness * pulse * 0.6;
        finalColor = texel * fragColor;
    }
}