#version 150 core

in vec2 ourTexCoord;

out vec4 color;

uniform sampler2D ourTexture;

void main() {
  color = vec4(1.0, 1.0, 1.0, 1.0) * texture(ourTexture, ourTexCoord);
}
