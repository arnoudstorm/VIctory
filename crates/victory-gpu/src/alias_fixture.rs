use super::*;
use crate::host::{Host, HostList, compile};
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::core::{Interface, s};

const PRODUCE: &str = r#"
#define ROOT "RootConstants(num32BitConstants=2, b0), DescriptorTable(UAV(u0))"
cbuffer Params : register(b0) { uint epoch; uint mip; }
RWTexture2DArray<uint> atlas : register(u0);
[RootSignature(ROOT)]
[numthreads(1,1,1)]
void main(uint3 p : SV_DispatchThreadID) {
    atlas[p] = (24u << 27) | ((1u + epoch + p.x) << 18)
             | ((1u + mip * 32u + p.y) << 9) | (1u + p.z * 64u + p.x + p.y * 8u);
}
"#;

const CONSUME: &str = r#"
#define ROOT "RootConstants(num32BitConstants=1, b0), DescriptorTable(SRV(t0, numDescriptors=2)), UAV(u0)"
cbuffer Params : register(b0) { uint mip; }
Texture2DArray<uint> source : register(t0);
Texture2DArray<float3> sampled : register(t1);
RWByteAddressBuffer result : register(u0);
[RootSignature(ROOT)]
[numthreads(1,1,1)]
void main(uint3 p : SV_DispatchThreadID) {
    uint4 at = uint4(p, mip);
    uint index = (((mip * 2u + p.z) * 8u + p.y) * 8u + p.x) * 16u;
    result.Store4(index, uint4(source.Load(at), uint3(sampled.Load(at))));
}
"#;

const RESULT_BYTES: u64 = 3 * 2 * 8 * 8 * 16;

struct Resource(This);
impl Drop for Resource {
    fn drop(&mut self) {
        resource_release(self.0);
    }
}

fn at(address: u64, format: u32, flags: u32) -> Resource {
    let null = std::ptr::null();
    let mut value = null;
    let desc = ResourceDesc {
        flags,
        depth_or_array_size: 2,
        ..texture(8, 8, format, 3)
    };
    assert_eq!(create_at_address(null, address, &desc, 0, null.cast(), &RESOURCE_IIDS[0], &mut value), S_OK);
    Resource(value)
}

fn pipeline(host: &Host, shader: &str) -> (ID3D12RootSignature, ID3D12PipelineState) {
    let code = compile(shader, s!("main"), s!("cs_5_1")).expect("compile original fixture shader");
    // SAFETY: the compiled shader contains the root signature and the
    // bytecode and root signature live through pipeline creation.
    unsafe {
        let root: ID3D12RootSignature = host.device.CreateRootSignature(0, &code).unwrap();
        let state = host.device.CreateComputePipelineState(&D3D12_COMPUTE_PIPELINE_STATE_DESC {
            pRootSignature: std::mem::transmute_copy(&root),
            CS: D3D12_SHADER_BYTECODE { pShaderBytecode: code.as_ptr().cast(), BytecodeLength: code.len() },
            ..Default::default()
        }).unwrap();
        (root, state)
    }
}

struct Pass {
    heap: ID3D12DescriptorHeap,
    step: u32,
    produce: (ID3D12RootSignature, ID3D12PipelineState),
    consume: (ID3D12RootSignature, ID3D12PipelineState),
}

impl Pass {
    fn new(host: &Host, source: &Resource, view: &Resource) -> Self {
        let ty = D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV;
        // SAFETY: valid shader-visible heap and views for the two original
        // fixture textures. All five descriptors are immutable after setup.
        unsafe {
            let heap: ID3D12DescriptorHeap = host.device.CreateDescriptorHeap(&D3D12_DESCRIPTOR_HEAP_DESC {
                Type: ty, NumDescriptors: 5, Flags: D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE, NodeMask: 0,
            }).unwrap();
            let step = host.device.GetDescriptorHandleIncrementSize(ty);
            let cpu = |slot| D3D12_CPU_DESCRIPTOR_HANDLE {
                ptr: heap.GetCPUDescriptorHandleForHeapStart().ptr + (slot * step) as usize,
            };
            let source = host_resource(source.0).unwrap();
            let view = host_resource(view.0).unwrap();
            for mip in 0..3 {
                host.device.CreateUnorderedAccessView(source, None, Some(&D3D12_UNORDERED_ACCESS_VIEW_DESC {
                    Format: DXGI_FORMAT_R32_UINT,
                    ViewDimension: D3D12_UAV_DIMENSION_TEXTURE2DARRAY,
                    Anonymous: D3D12_UNORDERED_ACCESS_VIEW_DESC_0 {
                        Texture2DArray: D3D12_TEX2D_ARRAY_UAV { MipSlice: mip, ArraySize: 2, ..Default::default() },
                    },
                }), cpu(mip));
            }
            for (slot, texture, format) in [(3, source, DXGI_FORMAT_R32_UINT), (4, view, DXGI_FORMAT_R9G9B9E5_SHAREDEXP)] {
                host.device.CreateShaderResourceView(texture, Some(&D3D12_SHADER_RESOURCE_VIEW_DESC {
                    Format: format,
                    ViewDimension: D3D12_SRV_DIMENSION_TEXTURE2DARRAY,
                    Shader4ComponentMapping: D3D12_DEFAULT_SHADER_4_COMPONENT_MAPPING,
                    Anonymous: D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
                        Texture2DArray: D3D12_TEX2D_ARRAY_SRV { MipLevels: 3, ArraySize: 2, ..Default::default() },
                    },
                }), cpu(slot));
            }
            Self { heap, step, produce: pipeline(host, PRODUCE), consume: pipeline(host, CONSUME) }
        }
    }

    fn gpu(&self, slot: u32) -> D3D12_GPU_DESCRIPTOR_HANDLE {
        // SAFETY: the handle is from our shader-visible heap.
        let base = unsafe { self.heap.GetGPUDescriptorHandleForHeapStart() };
        D3D12_GPU_DESCRIPTOR_HANDLE { ptr: base.ptr + u64::from(slot * self.step) }
    }

    fn write(&self, list: &HostList, resource: &Resource, epoch: u32) {
        crate::list::ensure_state(list, resource.0, D3D12_RESOURCE_STATE_UNORDERED_ACCESS.0 as u32);
        let l = &list.list;
        // SAFETY: list holds the tracked texture, and immutable views and
        // pipelines survive the fence in the test.
        unsafe {
            l.SetDescriptorHeaps(&[Some(self.heap.clone())]);
            l.SetComputeRootSignature(&self.produce.0);
            l.SetPipelineState(&self.produce.1);
            for mip in 0..3 {
                let constants = [epoch, mip];
                l.SetComputeRoot32BitConstants(0, 2, constants.as_ptr().cast(), 0);
                l.SetComputeRootDescriptorTable(1, self.gpu(mip));
                l.Dispatch(8 >> mip, 8 >> mip, 2);
            }
        }
    }

    fn read(&self, list: &HostList, source: &Resource, view: &Resource, output: &ID3D12Resource) {
        const SRV: u32 = D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE.0 as u32;
        crate::list::ensure_state(list, source.0, SRV);
        crate::list::ensure_state(list, view.0, SRV);
        let l = &list.list;
        // SAFETY: the views, output and pipeline survive submission. Each
        // thread writes a distinct 16-byte element of the output UAV.
        unsafe {
            l.SetDescriptorHeaps(&[Some(self.heap.clone())]);
            l.SetComputeRootSignature(&self.consume.0);
            l.SetPipelineState(&self.consume.1);
            l.SetComputeRootDescriptorTable(1, self.gpu(3));
            l.SetComputeRootUnorderedAccessView(2, output.GetGPUVirtualAddress());
            for mip in 0..3 {
                l.SetComputeRoot32BitConstant(0, mip, 0);
                l.Dispatch(8 >> mip, 8 >> mip, 2);
            }
        }
    }
}

fn check(host: &Host, output: &ID3D12Resource, epoch: u32) {
    let bytes = host.copy_out_for_test(output, 0, RESULT_BYTES).expect("completed GPU readback");
    for mip in 0..3u32 {
        for layer in 0..2 {
            for y in 0..8 >> mip {
                for x in 0..8 >> mip {
                    let index = ((((mip * 2 + layer) * 8 + y) * 8 + x) * 16) as usize;
                    let word = |i: usize| u32::from_le_bytes(bytes[index + i * 4..index + i * 4 + 4].try_into().unwrap());
                    let (r, g, b) = (1 + layer * 64 + x + y * 8, 1 + mip * 32 + y, 1 + epoch + x);
                    assert_eq!(word(0), (24 << 27) | (b << 18) | (g << 9) | r,
                        "producer epoch {epoch}, layer {layer}, mip {mip}, xy {x},{y}");
                    assert_eq!([word(1), word(2), word(3)], [r, g, b],
                        "placed SRV epoch {epoch}, layer {layer}, mip {mip}, xy {x},{y}");
                }
            }
        }
    }
}

#[test]
#[ignore = "explicit headless D3D12 run on the measured adapter"]
fn synthetic_upload_before_alias() {
    let host = crate::host::host().expect("fixture requires a D3D12 adapter");
    let null = std::ptr::null();
    let mut initial = null;
    let desc = ResourceDesc { flags: 4, depth_or_array_size: 2, ..texture(8, 8, 42, 3) };
    assert_eq!(create_committed(null, &PROPERTIES, 0, &desc, 0, null.cast(), &RESOURCE_IIDS[0], &mut initial), S_OK);
    let initial = Resource(initial);
    let address = resource(initial.0).data() as u64;
    let view = at(address, 67, 0);
    resource(view.0).host.placed.store(true, std::sync::atomic::Ordering::Relaxed);
    let mut first_identity = 0;
    let record = host.new_list().unwrap();
    let repeat = host.new_list().unwrap();
    let mut measurements = Vec::new();
    for (index, epoch) in [11, 23, 35].into_iter().enumerate() {
        let replacement = (index == 2).then(|| at(address, 42, 4));
        let source = replacement.as_ref().unwrap_or(&initial);
        assert_eq!(alias_of(view.0), Some(source.0));
        let identity = host_resource(source.0).unwrap().as_raw() as usize;
        if index == 0 { first_identity = identity; }
        assert_eq!(identity == first_identity, index != 2);
        let setup = host.new_list().unwrap();
        let pass = Pass::new(host, source, &view);
        pass.write(&setup, source, epoch);
        setup.close();
        host.execute(&[&setup]);

        let output = host.uav_buffer(RESULT_BYTES).unwrap();
        let second = host.uav_buffer(RESULT_BYTES).unwrap();
        crate::texture::upload(&record, view.0, host_resource(view.0).unwrap());
        assert!(!record.first_use(view.0 as usize), "the earlier upload must have used the view");
        assert!(crate::texture::alias(&record, view.0, source.0));
        assert!(crate::texture::alias(&record, view.0, source.0));
        pass.read(&record, source, &view, &output);
        pass.write(&record, source, epoch + 1);
        assert!(crate::texture::alias(&record, view.0, source.0));
        pass.read(&record, source, &view, &second);
        record.close();

        let start = std::time::Instant::now();
        host.execute(&[&record]);
        measurements.push(start.elapsed().as_micros());
        check(host, &output, epoch);
        check(host, &second, epoch + 1);
        record.reset();

        // No competing upload in this recording. This is the control and
        // leaves both resources in readable states for the replacement.
        assert!(crate::texture::alias(&repeat, view.0, source.0));
        pass.read(&repeat, source, &view, &output);
        repeat.close();
        host.execute(&[&repeat]);
        check(host, &output, epoch + 1);
        repeat.reset();
        assert!(host.device_alive());
    }
    eprintln!("alias fixture adapter={:?} submit_and_wait_us={measurements:?} available_bytes={:?} dxgi_bytes={:?}",
        unsafe { host.device.GetAdapterLuid() }, crate::host::available_physical_ram(), host.video_memory());
}
