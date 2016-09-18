#version 150 core

in vec2 ourTexCoord;

out vec4 color;

uniform sampler2D ourTexture;

void main() {
  color = texture(ourTexture, ourTexCoord);
}
