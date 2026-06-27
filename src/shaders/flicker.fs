#version 330
in vec2 fragTexCoord;
in vec4 fragColor;
uniform sampler2D texture0;
uniform float time;
out vec4 finalColor;
void main() {
    vec4 texelColor = texture(texture0, fragTexCoord);

    float base_flicker = sin(time * 2.0) * 0.15 + sin(time * 5.7) * 0.10;
    float jitter = fract(sin(floor(time * 10.0) * 43758.5453));

    // Starts at 1.0, flicker pushes it up from there
    float intensity = max(1.0, 1.0 + base_flicker + jitter * 0.05);
    finalColor = texelColor * fragColor * intensity;
}