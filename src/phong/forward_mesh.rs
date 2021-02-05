
use crate::core::*;
use crate::objects::*;
use crate::lights::*;
use crate::phong::*;

pub struct PhongForwardMesh {
    context: Context,
    pub name: String,
    mesh: Mesh,
    pub material: PhongMaterial
}

impl PhongForwardMesh
{
    pub fn new(context: &Context, cpu_mesh: &CPUMesh, material: &PhongMaterial) -> Result<Self, Error>
    {
        if cpu_mesh.normals.is_none() {
            Err(Error::FailedToCreateMesh {message:
              "Cannot create a mesh without normals. Consider calling compute_normals on the CPUMesh before creating the mesh.".to_string()})?
        }
        unsafe {
            MESH_COUNT += 1;
        }
        Ok(Self {
            context: context.clone(),
            name: cpu_mesh.name.clone(),
            mesh: Mesh::new(context, cpu_mesh)?,
            material: material.clone()
        })
    }

    pub fn new_meshes(context: &Context, cpu_meshes: &[CPUMesh], materials: &[PhongMaterial]) -> Result<Vec<Self>, Error>
    {
        let mut meshes = Vec::new();
        for cpu_mesh in cpu_meshes {
            let material = cpu_mesh.material_name.as_ref().map(|material_name|
                materials.iter().filter(|m| &m.name == material_name).last()
                .map(|m| m.clone()).unwrap_or_else(|| PhongMaterial::default()))
                .unwrap_or_else(|| PhongMaterial::default());
            meshes.push(Self::new(context,cpu_mesh, &material)?);
        }
        Ok(meshes)
    }

    pub fn render_depth(&self, render_states: RenderStates, viewport: Viewport, transformation: &Mat4, camera: &camera::Camera) -> Result<(), Error>
    {
        self.render_with_ambient(render_states, viewport, transformation, camera, &AmbientLight::default())
    }

    pub fn render_with_ambient(&self, render_states: RenderStates, viewport: Viewport, transformation: &Mat4, camera: &camera::Camera, ambient_light: &AmbientLight) -> Result<(), Error>
    {
        let program = match self.material.color_source {
            ColorSource::Color(_) => {
                unsafe {
                    if PROGRAM_COLOR_AMBIENT.is_none()
                    {
                        PROGRAM_COLOR_AMBIENT = Some(Mesh::create_program(&self.context, include_str!("shaders/colored_forward_ambient.frag"))?);
                    }
                    PROGRAM_COLOR_AMBIENT.as_ref().unwrap()
                }
            },
            ColorSource::Texture(_) => {
                unsafe {
                    if PROGRAM_TEXTURE_AMBIENT.is_none()
                    {
                        PROGRAM_TEXTURE_AMBIENT = Some(Mesh::create_program(&self.context,include_str!("shaders/textured_forward_ambient.frag"))?);
                    }
                    PROGRAM_TEXTURE_AMBIENT.as_ref().unwrap()
                }
            }
        };
        program.add_uniform_vec3("ambientColor", &(ambient_light.color * ambient_light.intensity))?;

        match self.material.color_source {
            ColorSource::Color(ref color) => {
                program.add_uniform_vec4("surfaceColor", color)?;
            },
            ColorSource::Texture(ref texture) => {
                if !self.mesh.has_uvs() {
                    Err(Error::FailedToCreateMesh {message:"Cannot use a texture as color source without uv coordinates.".to_string()})?;
                }
                program.use_texture(texture.as_ref(),"tex")?;
            }
        }
        self.mesh.render(program, render_states, viewport,transformation, camera)
    }

    pub fn render_with_ambient_and_directional(&self, render_states: RenderStates, viewport: Viewport, transformation: &Mat4, camera: &camera::Camera, ambient_light: &AmbientLight, directional_light: &DirectionalLight) -> Result<(), Error>
    {
        let program = match self.material.color_source {
            ColorSource::Color(_) => {
                unsafe {
                    if PROGRAM_COLOR_AMBIENT_DIRECTIONAL.is_none()
                    {
                        PROGRAM_COLOR_AMBIENT_DIRECTIONAL = Some(Mesh::create_program(&self.context, &format!("{}\n{}",
                                                                                      &include_str!("shaders/light_shared.frag"),
                                                                                      &include_str!("shaders/colored_forward_ambient_directional.frag")))?);
                    }
                    PROGRAM_COLOR_AMBIENT_DIRECTIONAL.as_ref().unwrap()
                }
            },
            ColorSource::Texture(_) => {
                unsafe {
                    if PROGRAM_TEXTURE_AMBIENT_DIRECTIONAL.is_none()
                    {
                        PROGRAM_TEXTURE_AMBIENT_DIRECTIONAL = Some(Mesh::create_program(&self.context, &format!("{}\n{}",
                                                                                    include_str!("shaders/light_shared.frag"),
                                                                                    include_str!("shaders/textured_forward_ambient_directional.frag")))?)
                    }
                    PROGRAM_TEXTURE_AMBIENT_DIRECTIONAL.as_ref().unwrap()
                }
            }
        };
        program.add_uniform_vec3("ambientColor", &(ambient_light.color * ambient_light.intensity))?;

        program.add_uniform_vec3("eyePosition", &camera.position())?;
        program.use_texture(directional_light.shadow_map(), "shadowMap")?;
        program.use_uniform_block(directional_light.buffer(), "DirectionalLightUniform");
        self.material.bind(program, self.mesh.has_uvs())?;
        self.mesh.render(program, render_states, viewport, transformation, camera)
    }
}

impl Drop for PhongForwardMesh {

    fn drop(&mut self) {
        unsafe {
            MESH_COUNT -= 1;
            if MESH_COUNT == 0 {
                PROGRAM_COLOR_AMBIENT = None;
                PROGRAM_COLOR_AMBIENT_DIRECTIONAL = None;
                PROGRAM_TEXTURE_AMBIENT = None;
                PROGRAM_TEXTURE_AMBIENT_DIRECTIONAL = None;
            }
        }
    }
}

static mut PROGRAM_COLOR_AMBIENT: Option<Program> = None;
static mut PROGRAM_COLOR_AMBIENT_DIRECTIONAL: Option<Program> = None;
static mut PROGRAM_TEXTURE_AMBIENT: Option<Program> = None;
static mut PROGRAM_TEXTURE_AMBIENT_DIRECTIONAL: Option<Program> = None;
static mut MESH_COUNT: u32 = 0;
