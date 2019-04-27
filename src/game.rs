use wgpu::winit::{
    ElementState, Event, EventsLoop, KeyboardInput, VirtualKeyCode, Window, WindowEvent,
};

use crate::assets::Assets;
use crate::renderer::Renderer;
use cgmath::{Vector3, Zero};
use crate::model::Model;

pub fn run(title: &str, assets: &Assets) {
    let instance = wgpu::Instance::new();
    let adapter = instance.get_adapter(&wgpu::AdapterDescriptor {
        power_preference: wgpu::PowerPreference::HighPerformance,
    });
    let mut device = adapter.create_device(&wgpu::DeviceDescriptor {
        extensions: wgpu::Extensions {
            anisotropic_filtering: false,
        },
    });

    let mut events_loop = EventsLoop::new();
    let window = Window::new(&events_loop).unwrap();
    window.set_title(title);
    let size = window
        .get_inner_size()
        .unwrap()
        .to_physical(window.get_hidpi_factor());

    let surface = instance.create_surface(&window);
    let mut sc_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsageFlags::OUTPUT_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8Unorm,
        width: size.width.round() as u32,
        height: size.height.round() as u32,
    };
    let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);

    let mut renderer = Renderer::init(&sc_desc, &mut device, assets);
    renderer.add_model_group(&mut device, "cube", assets.models.find("cube").unwrap());
    renderer.add_model("cube", Model::new(Vector3::new(0.0, 0.0, 0.0)));
    renderer.add_model("cube", Model::new(Vector3::new(0.0, 2.0, 0.0)));

    let mut running = true;
    while running {
        events_loop.poll_events(|event| match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                let physical = size.to_physical(window.get_hidpi_factor());
                sc_desc.width = physical.width.round() as u32;
                sc_desc.height = physical.height.round() as u32;
                swap_chain = device.create_swap_chain(&surface, &sc_desc);
                renderer.resize(&sc_desc, &mut device);
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    input:
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        state: ElementState::Pressed,
                        ..
                    },
                    ..
                }
                | WindowEvent::CloseRequested => {
                    running = false;
                }
                WindowEvent::KeyboardInput {
                    input: KeyboardInput {
                        virtual_keycode: Some(key_code),
                        state: ElementState::Pressed,
                        ..
                    },
                    ..
                } => {
                    let mut movement: Vector3<f32> = Vector3::zero();
                    match key_code {
                        VirtualKeyCode::W => movement.y += 0.1,
                        VirtualKeyCode::A => movement.x -= 0.1,
                        VirtualKeyCode::S => movement.y -= 0.1,
                        VirtualKeyCode::D => movement.x += 0.1,
                        VirtualKeyCode::R => movement.z += 0.1,
                        VirtualKeyCode::F => movement.z -= 0.1,
                        _ => (),
                    }
                    if !movement.is_zero() {
                        renderer.move_camera(&mut device, movement);
                    }
                }
                _ => {}
            },
            _ => (),
        });

        let frame = swap_chain.get_next_texture();
        renderer.render(&frame, &mut device);
    }
}
