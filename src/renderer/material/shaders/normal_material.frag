
in vec3 nor;

#ifdef USE_TEXTURE
in vec3 tang;
in vec3 bitang;
uniform sampler2D normalTexture;
uniform float normalScale;
#endif

layout (location = 0) out vec4 outColor;

void main()
{

    vec3 normal = normalize(gl_FrontFacing ? nor : -nor);
#ifdef USE_TEXTURE
    vec3 tangent = normalize(gl_FrontFacing ? tang : -tang);
    vec3 bitangent = normalize(gl_FrontFacing ? bitang : -bitang);
    mat3 tbn = mat3(tangent, bitangent, normal);
    normal = tbn * ((2.0 * texture(normalTexture, uvs).xyz - 1.0) * vec3(normalScale, normalScale, 1.0));
#endif
    outColor = vec4(0.5 + 0.5 * normal, 1.0);
}