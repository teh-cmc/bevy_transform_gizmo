use std::num::NonZeroU64;

use bevy::{
    ecs::system::{lifetimeless::SRes, SystemParamItem},
    pbr::MaterialPipeline,
    prelude::*,
    reflect::TypeUuid,
    render::{
        render_asset::{PrepareAssetError, RenderAsset},
        render_resource::{
            encase, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Buffer,
            BufferBindingType, BufferInitDescriptor, BufferUsages, ShaderStages, ShaderType,
        },
        renderer::RenderDevice,
    },
};

pub const GIZMO_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 13953800272683943019);

#[derive(Debug, Clone, Default, TypeUuid)]
#[uuid = "0cf245a7-ce7a-4473-821c-111e6f359193"]
pub struct GizmoMaterial {
    pub color: Color,
}
impl From<Color> for GizmoMaterial {
    fn from(color: Color) -> Self {
        GizmoMaterial { color }
    }
}

#[derive(Clone, Default, ShaderType)]
pub struct GizmoMaterialUniformData {
    pub color: Vec4,
}
const GIZMO_MATERIAL_UBO_SIZE: NonZeroU64 = GizmoMaterialUniformData::METADATA.min_size().0;

#[derive(Clone)]
pub struct GpuGizmoMaterial {
    _buffer: Buffer,
    bind_group: BindGroup,
}

impl RenderAsset for GizmoMaterial {
    type ExtractedAsset = GizmoMaterial;
    type PreparedAsset = GpuGizmoMaterial;
    type Param = (SRes<RenderDevice>, SRes<MaterialPipeline<Self>>);
    fn extract_asset(&self) -> Self::ExtractedAsset {
        self.clone()
    }

    fn prepare_asset(
        material: Self::ExtractedAsset,
        (render_device, material_pipeline): &mut SystemParamItem<Self::Param>,
    ) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>> {
        let value = GizmoMaterialUniformData {
            color: material.color.as_linear_rgba_f32().into(),
        };

        let mut buffer = encase::UniformBuffer::new([0u8; GIZMO_MATERIAL_UBO_SIZE.get() as usize]);
        buffer.write(&value).unwrap();

        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            contents: buffer.as_ref(),
            label: None,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: None,
            layout: &material_pipeline.material_layout,
        });

        Ok(GpuGizmoMaterial {
            _buffer: buffer,
            bind_group,
        })
    }
}

impl Material for GizmoMaterial {
    fn fragment_shader(_asset_server: &AssetServer) -> Option<Handle<Shader>> {
        Some(GIZMO_SHADER_HANDLE.typed())
    }

    fn vertex_shader(_asset_server: &AssetServer) -> Option<Handle<Shader>> {
        Some(GIZMO_SHADER_HANDLE.typed())
    }

    fn alpha_mode(_material: &<Self as RenderAsset>::PreparedAsset) -> AlphaMode {
        AlphaMode::Blend
    }

    fn bind_group(render_asset: &<Self as RenderAsset>::PreparedAsset) -> &BindGroup {
        &render_asset.bind_group
    }

    fn bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
        render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(GIZMO_MATERIAL_UBO_SIZE),
                },
                count: None,
            }],
            label: None,
        })
    }
}
